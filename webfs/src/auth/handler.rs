use axum::{
    extract::{Request, State, OriginalUri},
    response::{IntoResponse, Response},
    http::{StatusCode, Method, Uri, header::{ HeaderMap}},
    response::Json,
};
use url::Origin;
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
) -> Result<Json<AuthIdentity>, (StatusCode, Json<serde_json::Value>)> {
    let auth_uri = headers.get("x-original-uri").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    let uri = Uri::try_from(auth_uri.unwrap_or(uri.to_string())).unwrap_or(uri);
    let mut auth_request = AuthRequest::new(&uri, "GET", &headers);
    let auth_request_clone = auth_request.clone();
    match keycloak::check_auth(&state, &auth_request_clone).await {
        Ok(auth_identity) => match auth_identity {
            AuthIdentity::Claims(claims) => {
                tracing::info!("Nginx auth success for {}", claims.default_webdavfs.clone().unwrap_or_default());
                Ok(Json(AuthIdentity::Claims(claims)))
            }
            AuthIdentity::FileSysID(fs_id) => {
                tracing::info!("Nginx auth success for {}", fs_id);
                Ok(Json(AuthIdentity::FileSysID(fs_id)))
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
