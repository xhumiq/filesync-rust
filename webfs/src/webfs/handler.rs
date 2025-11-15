use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Json,
};
use crate::models::files::*;
use crate::auth::handler::with_auth;


pub async fn list_files_handler(
    state: State<crate::AppState>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Channel>, StatusCode> {
    with_auth(&state, &headers, |_claims| {
        let state = state.clone();
        let path = format!("{}/{}", state.base_path, path);
        async move {
            const lang: &str = "en";
            tracing::info!("Listing files for path: {} {}", lang, path);

            let channel = state.config.clone().get_folder_info(lang, &path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let entries = Channel::read_dir(&channel).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let mut channel = channel;
            channel.set_entries(entries, None);
            Ok(Json(channel))
        }
    })
    .await
}
