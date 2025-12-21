use axum::{http::{StatusCode, Uri}, response::Json};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use openssl::string;
use reqwest::Client;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use moka::future::Cache;
use std::time::{Duration, Instant};
use tracing;

use crate::models::{auth::*, files::FolderShare};

struct CachedJWKS {
  jwks: JWKS,
  fetched_at: Instant,
}

lazy_static! {
  static ref JWKS_CACHE: Arc<RwLock<Option<CachedJWKS>>> = Arc::new(RwLock::new(None));
  pub static ref SIGNING_KEYS: Arc<RwLock<SigningKeys>> = Arc::new(RwLock::new(SigningKeys::new(3600 * 24 * 30, 3600)));
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn get_jwks(keycloak_url: &str, realm: &str, http_client: &Client) -> Result<JWKS, StatusCode> {
    // Check cache
  {
    let cache = JWKS_CACHE.read().await;
    if let Some(cached) = &*cache {
      if cached.fetched_at.elapsed() < Duration::from_secs(14400) { // 4 hours
        return Ok(cached.jwks.clone());
      }
    }
  }

    // Fetch new
    let jwks_url = format!("{}/realms/{}/protocol/openid-connect/certs", keycloak_url, realm);
    let jwks_response = http_client
        .get(&jwks_url)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jwks: JWKS = jwks_response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update cache
    let mut cache = JWKS_CACHE.write().await;
    *cache = Some(CachedJWKS {
        jwks: jwks.clone(),
        fetched_at: Instant::now(),
    });

    Ok(jwks)
}

pub async fn authenticate(
    state: crate::AppState,
    auth_req: BasicAuthRequest,
    http_client: &Client,
    passwd: Cache<String, AuthResponse>,
    tokens: Cache<String, AuthResponse>
) -> Result<AuthResponse, (StatusCode, String)> {
    if auth_req.use_cache{
        let key = format!("{}:{}", &auth_req.username, &auth_req.password);
        if let Some(auth) = passwd.get(&key).await {
            return Ok(auth);
        }
    }

    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_url, state.realm
    );

    let mut params = HashMap::new();
    params.insert("client_id", state.client_id.to_string());
    params.insert("client_secret", state.client_secret.to_string());
    params.insert("grant_type", "password".to_string());
    params.insert("username", auth_req.username.clone());
    params.insert("password", auth_req.password.clone());
    params.insert("scope", "openid".to_string());

    tracing::debug!("Login attempt {} for user: {}", &token_url, auth_req.username);
    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            tracing::debug!("Error sending auth request: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send authentication request: {}", e))
        })?;

    if response.status().is_success() {
        let token: TokenResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::debug!("Error parsing token response: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse token response: {}", e))
            })?;

        // Decode claims
        let claims = decode_jwt_payload_struct(&token.access_token)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to decode JWT claims: {}", e)))?;
        tracing::debug!("Login successful for user: {}", auth_req.username);

        // Calculate expiration dates
        let now = Utc::now();
        let expires_at = (now + chrono::Duration::seconds(token.expires_in as i64)).to_rfc3339();
        let refresh_expires_at = (now + chrono::Duration::seconds(token.refresh_expires_in as i64)).to_rfc3339();

        let mut folder: Option<FolderShare> = None;
        if let Some(ref fs_id) = claims.default_webdavfs {
            if !fs_id.is_empty(){
                folder = state.config.folders.get(fs_id).cloned();
                if folder.is_none() {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Folder {} not found", fs_id)));
                }                
            }
        }
        let resp = AuthResponse {
            jwt_token: token.access_token.clone(),
            refresh_token: token.refresh_token,
            token_hash: hash_token(&token.access_token),
            expires_at,
            refresh_expires_at,
            claims,
            folder,
        };
        passwd.insert(format!("{}:{}", &auth_req.username, &auth_req.password), resp.clone()).await;
        tokens.insert(resp.token_hash.clone(), resp.clone()).await;
        tokens.insert(resp.jwt_token.clone(), resp.clone()).await;
        Ok(resp)
    } else {
        let body = response.text().await.unwrap_or_else(|_| "Failed to read response body".to_string());
        let error_msg = if let Ok(keycloak_err) = serde_json::from_str::<KeycloakError>(&body) {
            keycloak_err.error_description.unwrap_or_else(|| body.clone())
        } else {
            body.clone()
        };
        tracing::debug!("Login invalid for user: {}, response body: {}", auth_req.username, body);
        Err((StatusCode::UNAUTHORIZED, error_msg))
    }
}

pub async fn verify_token(
    keycloak_url: &str,
    realm: &str,
    token: &str,
    http_client: &Client,
) -> Result<bool, StatusCode> {
    let jwks = get_jwks(keycloak_url, realm, http_client).await?;

    // Decode header to get kid
    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;
    tracing::debug!("Verify JWT Token Kid: {}", kid);

    // Find the key
    let key = jwks.keys.into_iter().find(|k| k.kid == kid).ok_or(StatusCode::UNAUTHORIZED)?;

    // Create decoding key
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&format!("{}/realms/{}", keycloak_url, realm)]);
    validation.set_audience(&["account"]);
    validation.validate_exp = false; // We check exp manually

    tracing::debug!("Validation Start: {}", format!("{}/realms/{}", keycloak_url, realm));
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            // Check expiration manually
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .as_secs() as u64;
            tracing::debug!("Auth Token Data Claims Exp: {} {}", token_data.claims.exp.to_string(), token_data.claims.exp > now);
            if token_data.claims.exp > now {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            tracing::debug!("Token verification error: {}", e);
            // If signature validation failed, refresh JWKS and try again
            if matches!(e.kind(), jsonwebtoken::errors::ErrorKind::InvalidSignature) {
                // Clear cache
                {
                    let mut cache = JWKS_CACHE.write().await;
                    *cache = None;
                }
                // Get fresh JWKS
                let fresh_jwks = get_jwks(keycloak_url, realm, http_client).await?;
                let fresh_key = fresh_jwks.keys.into_iter().find(|k| k.kid == kid).ok_or(StatusCode::UNAUTHORIZED)?;
                let fresh_decoding_key = DecodingKey::from_rsa_components(&fresh_key.n, &fresh_key.e).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                match decode::<Claims>(token, &fresh_decoding_key, &validation) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
    }
}

pub async fn refresh_token(
    state: crate::AppState,
    refresh_req: RefreshRequest,
    http_client: &Client,
    passwd: Cache<String, AuthResponse>,
    tokens: Cache<String, AuthResponse>
) -> Result<AuthResponse, (StatusCode, String)> {
    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_url, state.realm
    );

    let mut params = HashMap::new();
    params.insert("client_id", state.client_id.to_string());
    params.insert("client_secret", state.client_secret.to_string());
    params.insert("grant_type", "refresh_token".to_string());
    params.insert("refresh_token", refresh_req.refresh_token);

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send refresh request".to_string()))?;

    if response.status().is_success() {
        let token: TokenResponse = response
            .json()
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse refresh response".to_string()))?;

        // Decode claims
        let claims = decode_jwt_payload_struct(&token.access_token)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to decode JWT claims".to_string()))?;

        // Calculate expiration dates
        let now = Utc::now();
        let expires_at = (now + chrono::Duration::seconds(token.expires_in as i64)).to_rfc3339();
        let refresh_expires_at = (now + chrono::Duration::seconds(token.refresh_expires_in as i64)).to_rfc3339();

        let mut folder: Option<FolderShare> = None;
        if let Some(ref fs_id) = claims.default_webdavfs {
            if !fs_id.is_empty(){
                folder = state.config.folders.get(fs_id).cloned();
                if folder.is_none() {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Folder {} not found", fs_id)));
                }                
            }
        }
        let resp = AuthResponse {
            jwt_token: token.access_token.clone(),
            refresh_token: token.refresh_token,
            token_hash: hash_token(&token.access_token),
            expires_at,
            refresh_expires_at,
            claims,
            folder,
        };
        tokens.insert(resp.token_hash.clone(), resp.clone()).await;
        tokens.insert(resp.jwt_token.clone(), resp.clone()).await;
        Ok(resp)
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid refresh token".to_string()))
    }
}

pub fn decode_jwt_payload_struct(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT".into());
    }
    let payload = parts[1];
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine.decode(payload)?;
    let claims: Claims = serde_json::from_slice(&decoded)?;
    Ok(claims)
}

pub async fn check_auth(state: &crate::AppState, request: &AuthRequest, passwd: Cache<String, AuthResponse>, tokens: Cache<String, AuthResponse>) -> 
    Result<AuthInfo, (StatusCode, Json<serde_json::Value>)>{
    if !request.jwt_token.is_none() {
        let jwt_token = request.jwt_token.as_ref().unwrap().clone();
        tracing::debug!("Auth JWT token: {}", &jwt_token);
        if let Some(auth) = tokens.get(&jwt_token).await {
            return Ok(AuthInfo::FromAuth(auth));
        }
        let active = verify_token(
            &state.keycloak_url,
            &state.realm,
            &request.jwt_token.as_ref().unwrap(),
            &state.http_client,
        )
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "token verification failed"}))))?;
        
        if !active {
            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "token inactive"}))));
        }

        let claims = decode_jwt_payload_struct(&jwt_token)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "failed to decode claims"}))))?;
        let folder = if let Some(ref fs_id) = claims.default_webdavfs {
            state.config.folders.get(fs_id).cloned()
        } else {
            state.config.folders.get("default").cloned()
        };
        return Ok(AuthInfo::new(claims, folder));
    }
    let basic_auth = request.basic_auth();
    if let Some(basic_auth) = basic_auth {
        tracing::debug!("Keycloak auth: {} / {}", &basic_auth.username, &basic_auth.password);
        match authenticate(
            state.clone(),
            basic_auth,
            &state.http_client,
            state.passwd.clone(),
            state.tokens.clone(),
        ).await {
            Ok(auth_resp) => {
                return Ok(AuthInfo::FromAuth(auth_resp))
            }
            Err((status, msg)) => {
                tracing::debug!("Keycloak Failed: {} / {}", status.as_str(), msg);
                return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Keycloak authentication failed", "status": status.as_u16(), "msg": msg}))));
            }
        }
    }
    if let (Some(uri), Some(method)) = (request.url.as_ref(), request.method.as_ref()) {
        if uri.contains("key_id="){
            let signing_keys = SIGNING_KEYS.clone();
            let uri_obj = Uri::try_from(uri).unwrap();
            tracing::debug!("Verify signurl attempt {} for user: {}", &method, uri.clone());
            match SignUrlResponse::from_url(&method, uri){
                Ok(resp) => {
                    let signing_keys = signing_keys.read().await;
                    match signing_keys.verify_signed_url(&resp).await {
                        Ok(_) => {
                            let query = uri_obj.query().unwrap_or("");
                            drop(signing_keys);
                            let auth = tokens.get(&resp.tid).await
                                .map(|auth| Ok(AuthInfo::FromAuth(auth)))
                                .unwrap_or(Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "token verification failed"})))))?;
                            return Ok(auth)
                        }
                        Err(e) => {
                            tracing::debug!("Bad signurl: {} / {}", &resp.tid, &resp.key_id);
                            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": e.to_string()}))))
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("Invalid signurl: {} / {}", &method, &uri);
                    return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": e.to_string()}))))
                }
            }
        }
    }
    return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))));
}