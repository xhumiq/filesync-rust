use axum::{http::{StatusCode, Uri}, response::Json};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use tracing;

use crate::models::{auth::*, files::FolderShare};

struct CachedJWKS {
  jwks: JWKS,
  fetched_at: Instant,
}

lazy_static! {
  static ref JWKS_CACHE: Arc<RwLock<Option<CachedJWKS>>> = Arc::new(RwLock::new(None));
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
) -> Result<AuthResponse, (StatusCode, String)> {
    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.keycloak_url, state.realm
    );

    let mut params = HashMap::new();
    params.insert("client_id", state.client_id.to_string());
    params.insert("client_secret", state.client_secret.to_string());
    params.insert("grant_type", "password".to_string());
    params.insert("username", auth_req.username.clone());
    params.insert("password", auth_req.password);
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
        Ok(AuthResponse {
            jwt_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at,
            refresh_expires_at,
            claims,
            folder,
        })
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
            tracing::debug!("Auth Token Data Claims Exp: {}", token_data.claims.exp.to_string());
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
        Ok(AuthResponse {
            jwt_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at,
            refresh_expires_at,
            claims,
            folder,
        })
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

pub async fn check_auth(state: &crate::AppState, request: &AuthRequest) -> Result<AuthIdentity, (StatusCode, Json<serde_json::Value>)>{
    if !request.jwt_token.is_none() {
        let jwt_token = request.jwt_token.as_ref().unwrap();
        tracing::debug!("Auth JWT token: {}", &jwt_token);
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
        return Ok(AuthIdentity::Claims(claims))
    }
    let basic_auth = request.basic_auth();
    if let Some(basic_auth) = basic_auth {
        match authenticate(
            state.clone(),
            basic_auth,
            &state.http_client,
        ).await {
            Ok(auth_resp) => {
                return Ok(AuthIdentity::Claims(auth_resp.claims))
            }
            Err((status, msg)) => {
                return Err((status, Json(serde_json::json!({"error": msg}))))
            }
        }
    }
    if let (Some(uri), Some(method)) = (request.url.as_ref(), request.method.as_ref()) {
        let uri_obj = Uri::try_from(uri).unwrap();
        match SignUrlResponse::from_url(&method, uri){
            Ok(resp) => {
                match state.signing_keys.lock().unwrap().verify_signed_url(&resp) {
                    Ok(_) => {
                        let query = uri_obj.query().unwrap_or("");
                        let fs_id = query.split('&').find(|p| p.starts_with("fs_id=")).and_then(|p| p.split('=').nth(1)).unwrap_or("").to_string();
                        return Ok(AuthIdentity::FileSysID(fs_id))
                    }
                    Err(e) => {
                        return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": e.to_string()}))))
                    }
                }
            }
            Err(e) => {
                return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": e.to_string()}))))
            }
        }
    }
    return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))));
}