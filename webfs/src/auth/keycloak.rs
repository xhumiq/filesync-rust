use crate::models::auth::*;
use axum::http::StatusCode;
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
    keycloak_url: &str,
    realm: &str,
    client_id: &str,
    client_secret: &str,
    auth_req: AuthRequest,
    http_client: &Client,
) -> Result<AuthResponse, StatusCode> {
    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        keycloak_url, realm
    );

    let mut params = HashMap::new();
    params.insert("client_id", client_id.to_string());
    params.insert("client_secret", client_secret.to_string());
    params.insert("grant_type", "password".to_string());
    params.insert("username", auth_req.username.clone());
    params.insert("password", auth_req.password);
    params.insert("scope", "openid".to_string());

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status().is_success() {
        let token: TokenResponse = response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Decode claims
        let claims = decode_jwt_payload_struct(&token.access_token)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        println!("TRACE: Login successful for user: {}", auth_req.username);

        // Calculate expiration dates
        let now = Utc::now();
        let expires_at = (now + chrono::Duration::seconds(token.expires_in as i64)).to_rfc3339();
        let refresh_expires_at = (now + chrono::Duration::seconds(token.refresh_expires_in as i64)).to_rfc3339();

        Ok(AuthResponse {
            jwt_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at,
            refresh_expires_at,
            claims,
        })
    } else {
        Err(StatusCode::UNAUTHORIZED)
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

    // Find the key
    let key = jwks.keys.into_iter().find(|k| k.kid == kid).ok_or(StatusCode::UNAUTHORIZED)?;

    // Create decoding key
    let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[&format!("{}/realms/{}", keycloak_url, realm)]);
    validation.set_audience(&["account"]);
    validation.validate_exp = false; // We check exp manually

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            // Check expiration manually
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u64;
            if token_data.claims.exp > now {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
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
    keycloak_url: &str,
    realm: &str,
    client_id: &str,
    client_secret: &str,
    refresh_req: RefreshRequest,
    http_client: &Client,
) -> Result<AuthResponse, StatusCode> {
    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        keycloak_url, realm
    );

    let mut params = HashMap::new();
    params.insert("client_id", client_id.to_string());
    params.insert("client_secret", client_secret.to_string());
    params.insert("grant_type", "refresh_token".to_string());
    params.insert("refresh_token", refresh_req.refresh_token);

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status().is_success() {
        let token: TokenResponse = response
            .json()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Decode claims
        let claims = decode_jwt_payload_struct(&token.access_token)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Calculate expiration dates
        let now = Utc::now();
        let expires_at = (now + chrono::Duration::seconds(token.expires_in as i64)).to_rfc3339();
        let refresh_expires_at = (now + chrono::Duration::seconds(token.refresh_expires_in as i64)).to_rfc3339();

        Ok(AuthResponse {
            jwt_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_at,
            refresh_expires_at,
            claims,
        })
    } else {
        Err(StatusCode::UNAUTHORIZED)
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

fn decode_jwt_payload(token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT".into());
    }
    let payload = parts[1];
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine.decode(payload)?;
    let claims: Value = serde_json::from_slice(&decoded)?;
    Ok(serde_json::to_string_pretty(&claims)?)
}