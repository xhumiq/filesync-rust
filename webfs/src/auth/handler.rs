use axum::{
    extract::{State, OriginalUri},
    response::{IntoResponse, Response},
    http::{StatusCode, Method, Uri, header::{ HeaderMap, HeaderValue}},
    response::Json,
};
use crate::models::auth::*;
use crate::auth::keycloak;

pub async fn authenticate_handler(
    State(state): State<crate::AppState>,
    Json(auth_req): Json<BasicAuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let response = keycloak::authenticate(
        state.clone(),
        auth_req,
        &state.http_client,
    )
    .await
    .map_err(|(status, msg)| (status, Json(serde_json::json!({"error": msg}))))?;
    Ok(Json(response))
}

pub async fn signurl_handler(
    State(state): State<crate::AppState>,
    OriginalUri(uri): OriginalUri,
    method: Method,
    headers: HeaderMap,
    Json(request): Json<SignUrlRequest>,
) -> Result<Json<SignUrlResponse>, (StatusCode, Json<serde_json::Value>)> {
    let auth_request = AuthRequest::new(&uri, method.as_str(), &headers);
    match keycloak::check_auth(&state, &auth_request).await {
        Ok(_auth_identity) => {
            let mut signing_keys = state.signing_keys.lock().unwrap();
            let response = signing_keys.generate_signed_url(&request)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
            Ok(Json(response))
        },
        Err((status, msg)) => {
            tracing::info!("Signurl auth failed for {}", auth_request.url.as_ref().unwrap().clone());
            Err((status, msg))
        }
    }
}

pub async fn nginx_handler(
    State(state): State<crate::AppState>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let method = headers.get("x-original-method").and_then(|h| h.to_str().ok()).unwrap_or("GET");
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("");

    let is_webdav = is_webdav(method, user_agent);

    tracing::info!("Nginx auth check: method={}, user_agent={}, is_webdav={}", method, user_agent, is_webdav);

    if !is_webdav {
        let auth_uri = headers.get("x-original-uri").and_then(|h| h.to_str().ok()).unwrap_or("");
        if auth_uri.starts_with("/css/") || auth_uri.starts_with("/images/") || auth_uri.starts_with("/js/") || auth_uri.starts_with("/fonts/") || auth_uri.starts_with("/assets/") || (auth_uri.starts_with("/") && !auth_uri[1..].contains('/') && auth_uri.contains('.')) {
            tracing::info!("Skipping auth for non-WebDAV CSR request");
            return Ok(Response::builder().status(200).body("".into()).unwrap());
        }
        if auth_uri.starts_with("/auth/") || auth_uri.starts_with("/fs/") {
            tracing::info!("Skipping auth for non-WebDAV API request");
            return Ok(Response::builder().status(200).body("".into()).unwrap());
        }
        if auth_uri == "/" {
            tracing::info!("Skipping auth for CSR request");
            return Ok(Response::builder().status(200).body("".into()).unwrap());
        }
    }

    let auth_uri = headers.get("x-original-uri").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    let uri = Uri::try_from(auth_uri.unwrap_or(uri.to_string())).unwrap_or(uri);
    let auth_request = AuthRequest::new(&uri, method, &headers);
    let auth_request_clone = auth_request.clone();
    match keycloak::check_auth(&state, &auth_request_clone).await {
        Ok(auth_identity) => match auth_identity {
            AuthIdentity::Claims(claims) => {
                tracing::info!("Nginx auth success for {}", claims.default_webdavfs.clone().unwrap_or_default());
                let auth_identity = AuthIdentity::Claims(claims);
                let json = Json(auth_identity);
                let mut response = json.into_response();
                response.headers_mut().insert("X-Webdav-Socket", HeaderValue::from_static("media"));
                tracing::info!("Added x-webdav-socket header to response");
                Ok(response)
            }
            AuthIdentity::FileSysID(fs_id) => {
                tracing::info!("Nginx auth success for {}", fs_id);
                let auth_identity = AuthIdentity::FileSysID(fs_id);
                let json = Json(auth_identity);
                let mut response = json.into_response();
                response.headers_mut().insert("X-Webdav-Socket", HeaderValue::from_static("media"));
                tracing::info!("Added x-webdav-socket header to response");
                Ok(response)
            }
            AuthIdentity::None => {
                tracing::info!("Nginx auth failed for {}", auth_request.url.as_ref().unwrap().clone());
                Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "no token"}))))
            }
        },
        Err((status, msg)) => {
            tracing::info!("Nginx auth failed for {}", auth_request.url.as_ref().unwrap().clone());
            Err((status, msg))
        }
    }
}

pub fn is_webdav(method: &str, user_agent: &str) -> bool {
    match method {
        "PROPFIND" | "MKCOL" | "COPY" | "MOVE" | "LOCK" | "UNLOCK" | "OPTIONS" => true,
        _ => {
            user_agent.to_lowercase().contains("microsoft-webdav-miniredir") ||
            user_agent.to_lowercase().contains("webdavfs") ||
            user_agent.to_lowercase().contains("davclnt") ||
            user_agent.to_lowercase().contains("cyberduck") ||
            user_agent.to_lowercase().contains("winscp") ||
            user_agent.to_lowercase().contains("transmit") ||
            user_agent.to_lowercase().contains("webdrive") ||
            user_agent.to_lowercase().contains("bitkinex") ||
            user_agent.to_lowercase().contains("carotdav") ||
            user_agent.to_lowercase().contains("gvfs") ||
            user_agent.to_lowercase().contains("konqueror") ||
            user_agent.to_lowercase().contains("cadaver") ||
            user_agent.to_lowercase().contains("davfs") ||
            user_agent.to_lowercase().contains("litmus") ||
            user_agent.to_lowercase().contains("neon")
        }
    }
}

pub async fn refresh_handler(
    State(state): State<crate::AppState>,
    Json(refresh_req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let response = keycloak::refresh_token(
        state.clone(),
        refresh_req,
        &state.http_client,
    )
    .await
    .map_err(|(status, msg)| (status, Json(serde_json::json!({"error": msg}))))?;
    Ok(Json(response))
}
