use serde::{Deserialize, Serialize};
use super::channel::FolderShare;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub fn is_token_valid(token: &str) -> bool {
  let parts: Vec<&str> = token.split('.').collect();
  if parts.len() == 3 {
    let payload_b64 = parts[1];
    // JWT uses base64url, convert to base64
    let payload_b64 = payload_b64.replace('-', "+").replace('_', "/");
    // Add padding
    let payload_b64 = match payload_b64.len() % 4 {
      0 => payload_b64,
      2 => format!("{}==", payload_b64),
      3 => format!("{}=", payload_b64),
      _ => return false,
    };
    if let Ok(decoded_bytes) = base64::decode(&payload_b64) {
      if let Ok(payload_str) = std::str::from_utf8(&decoded_bytes) {
        if let Ok(payload_json) = serde_json::from_str::<serde_json::Value>(payload_str) {
          if let Some(exp) = payload_json.get("exp").and_then(|v| v.as_i64()) {
            let now = js_sys::Date::now() as i64 / 1000;
            return exp > now;
          }
        }
      }
    }
  }
  false
}
