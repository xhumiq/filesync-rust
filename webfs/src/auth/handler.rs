use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, Uri, request},
    response::{IntoResponse, Json, Response},
};
use base64::{Engine, engine::general_purpose};
use crate::models::auth::*;
use crate::auth::keycloak;

pub async fn authenticate_handler(
    State(state): State<crate::AppState>,
    Json(auth_req): Json<AuthRequest>,
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
    Json(sign_req): Json<SignUrlRequest>,
) -> Result<Json<SignUrlResponse>, (StatusCode, Json<serde_json::Value>)> {
    let headers = HeaderMap::new();
    with_auth(&state, &headers, "POST", None,async |identity| {
        let mut signing_keys = state.signing_keys.lock().unwrap();
        let response = signing_keys.generate_signed_url(&sign_req)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
        Ok(Json(response))
    }).await
}

pub async fn nginx_handler(
    State(state): State<crate::AppState>,
    request: Request,
) -> Response {
    let headers = request.headers();
    if let Some(auth) = headers.get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Basic ") {
                if let Ok(creds) = general_purpose::STANDARD.decode(&auth_str[6..]) {
                    if let Ok(creds_str) = String::from_utf8(creds) {
                        let parts: Vec<&str> = creds_str.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let auth_req = AuthRequest {
                                username: parts[0].to_string(),
                                password: parts[1].to_string(),
                            };
                            match keycloak::authenticate(
                                state.clone(),
                                auth_req,
                                &state.http_client,
                            )
                            .await {
                                Ok(_) => {
                                    tracing::info!("Basic auth success for user: {}", parts[0]);
                                    return (StatusCode::OK).into_response();
                                }
                                Err((status, msg)) => {
                                    tracing::error!("Authentication failed for {}: {}", request.uri(), msg);
                                    return (status, msg).into_response();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let url = request.headers().get("x-original-uri").and_then(|h| h.to_str().ok());
    if url.is_none() {
        tracing::error!("Authentication failed for {}", request.uri());
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let url = url.unwrap();
    let resp = SignUrlResponse::from_url("GET", url);
    match resp {
        Ok(resp) => {
            match state.signing_keys.lock().unwrap().verify_signed_url(&resp) {
                Ok(_) => {
                    tracing::info!("Signed URL auth success for {}", url);
                    return (StatusCode::OK).into_response();
                }
                Err(e) => {
                    tracing::error!("Url is invalid for {}: {}", url, e);
                    return (StatusCode::UNAUTHORIZED, e.to_string()).into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!("Authentication failed for {}: {}", url, e);
            return (StatusCode::UNAUTHORIZED, e.to_string()).into_response();
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

pub async fn with_auth<F, Fut, R>(state: &crate::AppState, headers: &HeaderMap, method: &str, uri: Option<&str>, f: F) -> Result<R, (StatusCode, Json<serde_json::Value>)>
where
    F: FnOnce(AuthIdentity) -> Fut,
    Fut: std::future::Future<Output = Result<R, (StatusCode, Json<serde_json::Value>)>>,
{
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if !auth_header.is_none() {
        let auth_header = auth_header.unwrap();
        let active = keycloak::verify_token(
            &state.keycloak_url,
            &state.realm,
            auth_header,
            &state.http_client,
        )
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "token verification failed"}))))?;

        if !active {
            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "token inactive"}))));
        }

        let claims = keycloak::decode_jwt_payload_struct(auth_header)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "failed to decode claims"}))))?;
        return f(AuthIdentity::Claims(claims)).await
    }
    let method_upper = method.to_uppercase();
    if let Some(uri) = uri {
        let uri_obj = Uri::try_from(uri).unwrap();
        match SignUrlResponse::from_url(&method_upper, uri){
            Ok(resp) => {
                match state.signing_keys.lock().unwrap().verify_signed_url(&resp) {
                    Ok(_) => {
                        let query = uri_obj.query().unwrap_or("");
                        let fsid = query.split('&').find(|p| p.starts_with("fsid=")).and_then(|p| p.split('=').nth(1)).unwrap_or("").to_string();
                        return f(AuthIdentity::FileSysID(fsid)).await
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