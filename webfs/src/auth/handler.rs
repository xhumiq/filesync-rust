use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use crate::models::auth::*;
use crate::auth::keycloak;

pub async fn authenticate_handler(
    State(state): State<crate::AppState>,
    Json(auth_req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let response = keycloak::authenticate(
        &state.keycloak_url,
        &state.realm,
        &state.client_id,
        &state.client_secret,
        auth_req,
        &state.http_client,
    )
    .await
    .map_err(|(status, msg)| (status, Json(serde_json::json!({"error": msg}))))?;
    Ok(Json(response))
}

pub async fn refresh_handler(
    State(state): State<crate::AppState>,
    Json(refresh_req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let response = keycloak::refresh_token(
        &state.keycloak_url,
        &state.realm,
        &state.client_id,
        &state.client_secret,
        refresh_req,
        &state.http_client,
    )
    .await?;
    Ok(Json(response))
}

pub async fn with_auth<F, Fut, R>(state: &State<crate::AppState>, headers: &HeaderMap, f: F) -> Result<R, StatusCode>
where
    F: FnOnce(Claims) -> Fut,
    Fut: std::future::Future<Output = Result<R, StatusCode>>,
{
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let active = keycloak::verify_token(
        &state.keycloak_url,
        &state.realm,
        auth_header,
        &state.http_client,
    )
    .await?;

    if !active {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let claims = keycloak::decode_jwt_payload_struct(auth_header)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    f(claims).await
}