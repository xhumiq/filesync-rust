use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    body::Body,
    Json,
};
use crate::models::files::*;
use std::path::Path as StdPath;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use mime_guess;
use chrono::Utc;
use serde_json;

pub async fn list_files_handler(
    state: State<crate::AppState>,
    Path(path): Path<String>,
    _headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    //with_auth(&state, &headers, |_claims| {
        let state = state.clone();
        let mut lang = "zh";
        let mut channel_opt: Option<Channel> = None;
        let mut full_path= String::new();

        if path.starts_with("zh/") || path.starts_with("en/") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                lang = parts[0];
                let channel_name = parts[1];
                if let Some(lang_map) = state.config.channels.get(lang) {
                    if let Some(ch) = lang_map.get(channel_name) {
                        channel_opt = Some(ch.clone());
                        full_path = ch.file_path.clone()
                    }
                }
            }
            if full_path.is_empty() {
                return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid path format"}))));
            }
        }

        if full_path.is_empty() {
            full_path = format!("{}/{}", state.base_path, path);
        }

        let path_obj = StdPath::new(&full_path);

        if !path_obj.exists() {
            tracing::error!("File not found: {}", full_path);
            return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "File not found"}))));
        }

        if path_obj.is_file() {
            let file = File::open(&full_path).await
                .map_err(|_| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "File not found"}))))?;

            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);
            let mut response = Response::new(body);

            let mime = mime_guess::from_path(&full_path).first_or_octet_stream();
            let content_type = mime.to_string().parse().map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to determine content type"}))))?;
            response.headers_mut().insert(header::CONTENT_TYPE, content_type);

            return Ok(response);
        } else if path_obj.is_dir() {
            // Continue with listing
            tracing::info!("Listing files for path: {} {}", lang, full_path);


            let channel = if let Some(ch) = channel_opt {
                ch
            } else {
                state.config.clone().get_folder_info(lang, &full_path).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to get folder info"}))))?
            };
            let cache_id = format!("{}/{}", channel.language, channel.name);

            // Check cache
            {
                let cache = state.channel_cache.lock().unwrap();
                if let Some((cached_channel, timestamp)) = cache.get(&cache_id) {
                    if Utc::now().signed_duration_since(*timestamp).num_seconds() < 300 {
                        tracing::info!("Using cached channel data for {}", cache_id);
                        return Ok(Json(cached_channel.clone()).into_response());
                    }
                }
            }

            let entries = Channel::read_dir(&channel).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to read directory"}))))?;
            let mut channel = channel;
            channel.set_entries(entries);
            // Cache the result
            {
                let mut cache = state.channel_cache.lock().unwrap();
                cache.insert(cache_id.to_string(), (channel.clone(), Utc::now()));
            }

            return Ok(Json(channel).into_response());
        } else {
            return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid request"}))));
        }
    //})
}

