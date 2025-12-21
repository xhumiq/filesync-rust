use axum::{
    extract::{State, OriginalUri, Request},
    response::{IntoResponse, Response},
    http::{StatusCode, Method, Uri, header::{ HeaderMap, HeaderValue}},
    response::Json,
    body::to_bytes,
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
        state.passwd.clone(),
        state.tokens.clone(),
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
    let signing_keys = keycloak::SIGNING_KEYS.clone();
    let auth_request = AuthRequest::new(&uri, method.as_str(), &headers);
    match keycloak::check_auth(&state, &auth_request, state.passwd.clone(), state.tokens.clone()).await {
        Ok(auth_identity) => {
            let response = {   
                let mut signing_keys = signing_keys.write().await;
                signing_keys.generate_signed_url(&request)
            }.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
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
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth_uri = headers.get("x-original-uri").and_then(|h| h.to_str().ok()).unwrap_or("/");
    let method = headers.get("x-original-method").and_then(|h| h.to_str().ok()).unwrap_or("GET");
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("");

    let is_webdav = is_webdav(method, user_agent);

    tracing::info!("Nginx auth check: method={}, user_agent={}, is_webdav={}", method, user_agent, is_webdav);
    
    if !is_webdav {
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

    let uri = Uri::try_from(auth_uri).unwrap_or(Uri::from_static("/"));
    let auth_request = AuthRequest::new(&uri, method, &headers);
    let auth_request_clone = auth_request.clone();
    match keycloak::check_auth(&state, &auth_request_clone, state.passwd.clone(), state.tokens.clone()).await {
        Ok(auth_identity) => {

            tracing::info!("auth_identity: {}", serde_json::to_string(&auth_identity).unwrap());
            let json = Json(auth_identity.clone());
            let mut response = json.into_response();
            let auth = auth_identity.folder.and_then(|f| Some(f.access_token()));
            response.headers_mut().insert("X-Webdav-Socket", HeaderValue::from_static("media"));
            if let Some(auth) = auth{
                response.headers_mut().insert("X-Socket-Auth", HeaderValue::from_str(&auth).unwrap());
            }
            //Ok(response)
            Ok(response)
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
            user_agent.to_lowercase().contains("webdav") ||
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
) -> Result<Json<Option<AuthResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let response = keycloak::refresh_token(
        state.clone(),
        refresh_req,
        &state.http_client,
        state.passwd.clone(),
        state.tokens.clone(),
    )
    .await
    .map_err(|(status, msg)| (status, Json(serde_json::json!({"error": msg}))))?;
    Ok(Json(Some(response)))
}
