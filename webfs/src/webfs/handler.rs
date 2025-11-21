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

pub async fn list_files_handler(
    state: State<crate::AppState>,
    Path(path): Path<String>,
    _headers: HeaderMap,
) -> impl IntoResponse {
    //with_auth(&state, &headers, |_claims| {
        let state = state.clone();
        let full_path = format!("{}/{}", state.base_path, path);
        let path_obj = StdPath::new(&full_path);

        if !path_obj.exists() {
            tracing::error!("File not found: {}", full_path);
            return Err(StatusCode::NOT_FOUND);
        }

        if path_obj.is_file() {
            let file = File::open(&full_path).await
                .map_err(|_| StatusCode::NOT_FOUND)?;

            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);
            let mut response = Response::new(body);

            let mime = mime_guess::from_path(&full_path).first_or_octet_stream();
            let content_type = mime.to_string().parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            response.headers_mut().insert(header::CONTENT_TYPE, content_type);

            return Ok(response);
        } else if path_obj.is_dir() {
            // Continue with listing
            const lang: &str = "en";
            tracing::info!("Listing files for path: {} {}", lang, full_path);

            let channel = state.config.clone().get_folder_info(lang, &full_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let cache_key = channel.name.clone();

            // Check cache
            {
                let cache = state.channel_cache.lock().unwrap();
                if let Some((cached_channel, timestamp)) = cache.get(&cache_key) {
                    if Utc::now().signed_duration_since(*timestamp).num_seconds() < 300 {
                        tracing::info!("Using cached channel data for {}", cache_key);
                        return Ok(Json(cached_channel.clone()).into_response());
                    }
                }
            }

            let entries = Channel::read_dir(&channel).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut channel = channel;
            channel.set_entries(entries);
            // for _entry in &mut channel.entries {
                // println!("Entry: {} {} {}", entry.file_date_stamp, entry.location, entry.event_code);
                // if entry.file_date_stamp == "251109" {
                //     println!("Entry: {} {} {}", entry.file_date_stamp, entry.location, entry.event_code);
                // }
            // }

            // Cache the result
            {
                let mut cache = state.channel_cache.lock().unwrap();
                cache.insert(cache_key.to_string(), (channel.clone(), Utc::now()));
            }

            return Ok(Json(channel).into_response());
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    //})
}

