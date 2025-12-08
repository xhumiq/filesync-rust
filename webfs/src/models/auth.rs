use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use super::files::FolderShare;
use hmac::{Hmac, Mac};
use nanoid::nanoid;
use sha2::Sha256;
use base64;
use url::Url;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Local, Utc};
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::collections::HashMap;
use base64::{Engine, engine::general_purpose};
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub refresh_expires_in: u32,
    pub refresh_token: Option<String>,
    pub token_type: String,
    #[serde(rename = "not-before-policy")]
    pub not_before_policy: u32,
    pub session_state: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub jwt_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: String,
    pub refresh_expires_at: String,
    pub claims: Claims,
    pub folder: Option<FolderShare>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntrospectResponse {
    active: bool,
    // other fields...
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakError {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JWKS {
    pub keys: Vec<JWK>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JWK {
    pub kid: String,
    pub n: String,
    pub e: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub kty: String,
    pub alg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub formatted: Option<String>,
    pub street_address: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccess {
    #[serde(flatten)]
    pub clients: std::collections::HashMap<String, ClientAccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub acr: Option<String>,
    pub address: Option<Address>,
    #[serde(rename = "allowed-origins")]
    pub allowed_origins: Option<Vec<String>>,
    pub aud: String,
    pub azp: Option<String>,
    pub default_webdavfs: Option<String>,
    #[serde(rename = "email_verified")]
    pub email_verified: Option<bool>,
    pub exp: u64,
    #[serde(rename = "family_name")]
    pub family_name: Option<String>,
    #[serde(rename = "given_name")]
    pub given_name: Option<String>,
    pub groups: Option<Vec<String>>,
    pub iat: u64,
    pub iss: String,
    pub jti: Option<String>,
    #[serde(rename = "preferred_username")]
    pub preferred_username: Option<String>,
    #[serde(rename = "resource_access")]
    pub resource_access: Option<ResourceAccess>,
    pub roles: Option<Vec<String>>,
    pub scope: Option<String>,
    #[serde(rename = "session_state")]
    pub session_state: Option<String>,
    pub sid: Option<String>,
    pub sub: String,
    pub typ: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUrlRequest {
    #[serde(default)]
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub fs_id: String,
    #[serde(default)]
    pub method: String,
}

impl SignUrlRequest {
    pub fn new(method: &str, url: &str) -> SignUrlRequest {
        SignUrlRequest{
            id: nanoid!().to_string(),
            url: url.to_string(),
            fs_id: String::new(),
            method: method.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignUrlResponse {
    pub id: String,
    pub url: String,
    pub fs_id: String,
    pub method: String,
    pub key_id: String,
    pub signature: String,
    pub expires_at: DateTime<Utc>,
}

impl SignUrlResponse {
    pub fn new(req: &SignUrlRequest) -> SignUrlResponse {
        SignUrlResponse{
            id: req.id.clone(),
            url: req.url.clone(),
            fs_id: String::new(),
            method: req.method.clone(),
            key_id: String::new(),
            signature: String::new(),
            expires_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        }
    }
    pub fn from_url(method: &str, url: &str) -> Result<SignUrlResponse> {
        let cleaned_url = HmacSigningKey::clean_url(&url);
        let url = Url::parse(&cleaned_url)?;
        let mut resp = SignUrlResponse{
            id: String::new(),
            url: url.to_string(),
            fs_id: String::new(),
            method: method.to_string(),
            key_id: String::new(),
            signature: String::new(),
            expires_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        for (key, value) in url.query_pairs() {
            let key = key.into_owned();
            let value = value.into_owned();
            if key == "signature" {
                resp.signature = value;
            } else if key == "expires" {
                let expires = value.parse::<i64>()?;
                resp.expires_at = DateTime::<Utc>::from_timestamp(expires, 0).ok_or(anyhow!("Invalid timestamp"))?;
            } else if key == "id" {
                resp.id = value;
            } else if key == "fs_id" {
                resp.fs_id = value;
            } else if key == "key_id" {
                resp.key_id = value;
            }
        }
        Ok(resp)
    }
}

#[derive(Debug)]
pub struct SigningKeys {
    pub keys: HashMap<String, HmacSigningKey>,
    pub cur_key: Option<HmacSigningKey>,
    pub last_create: DateTime<Local>,
    pub domain: String,
    pub key_expires_in_secs: u64,
    pub sig_expires_in_secs: u64,
}

impl SigningKeys {
    pub fn new(key_expires_in_secs: u64, sig_expires_in_secs: u64) -> SigningKeys {
        let mut sig_expires_in_secs = sig_expires_in_secs;
        if sig_expires_in_secs < 60 {
            sig_expires_in_secs = 60;
        }
        let mut key_expires_in_secs = key_expires_in_secs;
        if key_expires_in_secs < 60 {
            key_expires_in_secs = 60;
        }
        SigningKeys{
            keys: HashMap::new(),
            cur_key: None,
            domain: String::new(),
            last_create: Local::now().checked_sub_days(chrono::Days::new(365)).unwrap(),
            key_expires_in_secs: key_expires_in_secs,
            sig_expires_in_secs: sig_expires_in_secs,
        }
    }
    fn create_new_key(&mut self) {
        let mut key = HmacSigningKey::new(self.sig_expires_in_secs);
        key.set_domain(self.domain.clone());
        key.set_expires_at(Local::now().checked_add_signed(chrono::Duration::seconds(self.key_expires_in_secs as i64)).unwrap());
        self.keys.insert(key.key_id.clone(), key.clone());
        self.cur_key = Some(key);
    }
    fn current(&mut self) -> HmacSigningKey {
        if self.cur_key.is_none() || self.cur_key.as_ref().unwrap().is_expired() {
            self.create_new_key();
        }
        self.cur_key.as_ref().unwrap().clone()
    }
    pub fn generate_signed_url(&mut self, request: &SignUrlRequest) -> Result<SignUrlResponse> {
        let key = self.current();
        key.generate_signed_url(request)
    }
    pub fn verify_signed_url(&self, request: &SignUrlResponse) -> Result<url::Url> {
        match self.keys.get(&request.key_id){
            Some(key) => {
                if key.is_expired() {
                    return Err(anyhow!("Key is expired"));
                }
                key.verify_signed_url(request)
            },
            None => {
                return Err(anyhow!("Key not found"));
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HmacSigningKey {
    pub key_id: String,
    pub secret: [u8; 32],
    pub domain: String,
    pub expires_at: DateTime<Local>,
    pub expires_in_secs: u64,
}

impl HmacSigningKey {
    pub fn new(sig_exp_secs: u64) -> HmacSigningKey {
        let mut key = [0u8; 32];
        OsRng.try_fill_bytes(&mut key).unwrap();
        HmacSigningKey{
            key_id: nanoid!(),
            secret: key,
            domain: String::new(),
            expires_at: Local::now(),
            expires_in_secs: sig_exp_secs,
        }
    }
    pub fn is_expired(&self) -> bool {
        let now = Local::now();
        now > self.expires_at
    }
}

impl HmacSigningKey {
    pub fn set_key_id(&mut self, key_id: String) {
        self.key_id = key_id;
    }
    pub fn set_domain(&mut self, domain: String) {
        self.domain = domain;
    }
    pub fn set_expires_at(&mut self, expires_at: DateTime<Local>) {
        self.expires_at = expires_at;
    }
    pub fn set_expires_in_secs(&mut self, expires_in_secs: u64) {
        self.expires_in_secs = expires_in_secs;
    }

    pub fn clean_url(url: &str) -> String {
        let mut url = url.to_string();
        if url.ends_with('/') {
            url.pop();
        }
        if !url.starts_with("http"){
            let schema = if url.starts_with("localhost") { "http" } else { "https" }; // http or https schema
            if url.starts_with("//") {
                url = format!("{}:{}", schema, url);
            } else {
                url = format!("{}://{}", schema, url);
            }
        }
        url
    }

    pub fn trim_url(url: &str) -> String {
        let mut url = url.to_string();
        if url.ends_with('/') {
            url.pop();
        }
        if url.starts_with("http://") {
            url = url[7..].to_string();
        } else if url.starts_with("https://") {
            url = url[8..].to_string();
        }
        url
    }

    pub fn generate_signed_url(&self, request: &SignUrlRequest) -> Result<SignUrlResponse> {
        let mut request = (*request).clone();
        if request.id.is_empty() {
            request.id = nanoid!();
        }
        if request.method.is_empty() {
            request.method = "GET".to_string();
        }
        let cleaned_url = HmacSigningKey::clean_url(&request.url);
        let url = Url::parse(&cleaned_url)?;
        
        // Build canonical string: method + path + sorted query + expires
        let method = request.method.to_uppercase();
        let path = url.path();
        let expires = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + self.expires_in_secs;
        
        // Add expires to query params
        let mut url_with_expires = url.clone();
        url_with_expires.query_pairs_mut()
            .append_pair("expires", &expires.to_string())
            .append_pair("id", &request.id)
            .append_pair("key_id", &self.key_id);
        
        // Canonical string (deterministic order matters!)
        let canonical_query = url_with_expires
            .query_pairs()
            .filter(|(k, _)| k != "signature")  // Exclude signature itself
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        
        let full_query = if !canonical_query.is_empty() {
            format!("?{}", canonical_query)
        } else {
            String::new()
        };
        
        let string_to_sign = format!("{}{}{}", method, path, full_query);
        
        // 2. COMPUTE HMAC
        let mut mac = HmacSha256::new_from_slice(&self.secret)?;
        mac.update(string_to_sign.as_bytes());
        let signature = mac.finalize();
        let engine = general_purpose::STANDARD;
        let signature_b64 = engine.encode(signature.into_bytes());
        
        // 3. Append signature to URL
        url_with_expires.query_pairs_mut()
            .append_pair("signature", &signature_b64);

        let mut resp = SignUrlResponse::new(&request);
        resp.key_id = self.key_id.clone();
        resp.expires_at = DateTime::<Utc>::from_timestamp(expires as i64, 0).unwrap();
        resp.signature = signature_b64.clone();
        resp.url = url_with_expires.to_string();

        Ok(resp)
    }

    pub fn verify_signed_url(&self, request: &SignUrlResponse) -> Result<url::Url> {
        let engine = general_purpose::STANDARD;
        let cleaned_url = HmacSigningKey::clean_url(&request.url);
        let url = url::Url::parse(&cleaned_url)?;
        // Extract and validate signature
        let pairs = url.query_pairs();
        let mut signature_b64: Option<String> = None;
        if request.signature != "" {
            signature_b64 = Some(request.signature.clone());
        }
        let mut expires: Option<u64> = None;
        if request.expires_at != DateTime::<Utc>::from_timestamp(0, 0).unwrap() {
            expires = Some(request.expires_at.timestamp() as u64);
        }
        let mut canonical_parts: Vec<String> = Vec::new();
        
        for (key, value) in pairs {
            let key = key.into_owned();
            let value = value.into_owned();
            
            if key == "signature" {
                signature_b64 = Some(value);
            } else if key == "expires" {
                expires = Some(value.parse::<u64>()?);
                canonical_parts.push(format!("{}={}", key, value));
            } else {
                canonical_parts.push(format!("{}={}", key, value));
            }
        }
        
        let signature_b64 = signature_b64.ok_or(anyhow!("Missing signature"))?;
        let signature = engine.decode(&signature_b64)?;
        
        // Check expiration
        if let Some(exp) = expires {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            if now > exp {
                return Err(anyhow!("URL expired"));
            }
        }
        
        // Rebuild exact same string_to_sign
        let method = request.method.to_uppercase();
        let path = url.path();
        let query_string = canonical_parts.join("&");
        let full_query = if !query_string.is_empty() {
            format!("?{}", query_string)
        } else {
            String::new()
        };
        
        let string_to_sign = format!("{}{}{}", method, path, full_query);
        
        // Verify HMAC
        let mut mac = HmacSha256::new_from_slice(&self.secret)?;
        mac.update(string_to_sign.as_bytes());
        let expected_signature = mac.finalize().into_bytes();
        
        if signature != expected_signature.as_slice() {
            return Err(anyhow!("Invalid signature"));
        }
        
        Ok(url)
    }
}

pub enum AuthIdentity {
    Claims(Claims),
    FileSysID(String),
    None
}

impl AuthIdentity {
    pub fn from(claims: Claims) -> Self {
        AuthIdentity::Claims(claims)
    }
    pub fn from_file_sys_id(file_sys_id: String) -> Self {
        AuthIdentity::FileSysID(file_sys_id)
    }
    pub fn from_none() -> Self {
        AuthIdentity::None
    }
    pub fn get(&self) -> Option<AuthIdentity> {
        match self {
            AuthIdentity::Claims(claims) => Some(AuthIdentity::Claims(claims.clone())),
            AuthIdentity::FileSysID(file_sys_id) => Some(AuthIdentity::FileSysID(file_sys_id.clone())),
            AuthIdentity::None => None
        }
    }
    pub fn get_mut(&mut self) -> Option<AuthIdentity> {
        match self {
            AuthIdentity::Claims(claims) => Some(AuthIdentity::Claims(claims.clone())),
            AuthIdentity::FileSysID(file_sys_id) => Some(AuthIdentity::FileSysID(file_sys_id.clone())),
            AuthIdentity::None => None
        }
    }
    pub fn clone(&self) -> Self {
        match self {
            AuthIdentity::Claims(claims) => AuthIdentity::Claims(claims.clone()),
            AuthIdentity::FileSysID(file_sys_id) => AuthIdentity::FileSysID(file_sys_id.clone()),
            AuthIdentity::None => AuthIdentity::None
        }
    }
    pub fn as_ref(&self) -> Option<AuthIdentity> {
        match self {
            AuthIdentity::Claims(claims) => Some(AuthIdentity::Claims(claims.clone())),
            AuthIdentity::FileSysID(file_sys_id) => Some(AuthIdentity::FileSysID(file_sys_id.clone())),
            AuthIdentity::None => None
        }
    }
    pub fn as_mut(&mut self) -> Option<AuthIdentity> {
        match self {
            AuthIdentity::Claims(claims) => Some(AuthIdentity::Claims(claims.clone())),
            AuthIdentity::FileSysID(file_sys_id) => Some(AuthIdentity::FileSysID(file_sys_id.clone())),
            AuthIdentity::None => None
        }
    }
    pub fn file_sys_id(&self) -> String {
        match self {
            AuthIdentity::Claims(claims) => claims.default_webdavfs.clone().unwrap_or_default(),
            AuthIdentity::FileSysID(file_sys_id) => file_sys_id.clone(),
            AuthIdentity::None => String::new()
        }
    }
}
