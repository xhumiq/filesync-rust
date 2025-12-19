use webfs::auth::handler::{authenticate_handler, refresh_handler, signurl_handler, nginx_handler};
use webfs::models::auth::SigningKeys;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::{TcpListener, UnixListener};
use tokio::signal;
use webfs::AppState;
use reqwest::Client;
use tower_http::cors::CorsLayer;
use webfs::models::files::Channel;
use webfs::storage::Storage;
use webfs::webfs::handler::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(profile_file) = env::var("ENV_PROFILE") {
        println!("cargo:rerun-if-changed={}", profile_file);
        dotenvy::from_path(profile_file).ok();
    }

    if let Some(timestamp) = option_env!("VERGEN_BUILD_TIMESTAMP") {
        println!("Build Timestamp: {timestamp}");
    }
    if let Some(describe) = option_env!("VERGEN_GIT_DESCRIBE") {
        println!("git describe: {describe}");
    }
    let log_path = match std::env::var("LOG_FILE") {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("LOG_FILE not set: {}, using default", e);
            "../logs/webfs.log".to_string()
        }
    };

    webfs::init_tracing(log_path.as_str())?;

    let config_path = std::env::var("CONFIG_PATH").unwrap_or("config-test.yaml".to_string());
    tracing::info!("Application started Config: {}", config_path);

    let config = match Channel::read_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to read config from {}: {}", config_path, e);
            return Err(e.into());
        }
    };

    let keycloak_url = {
        let mut url = std::env::var("KEYCLOAK_URL").map_err(|e| {
            tracing::error!("KEYCLOAK_URL not set: {}", e);
            e
        })?;
        if !url.starts_with("http") {
            url = format!("https://{}", url);
        }
        url
    };

    let db_path = std::env::var("DB_PATH").unwrap_or("/srv/data/webfs/files.db".to_string());

    tracing::info!("Creating Database path: {}", db_path);

    let storage = match Storage::new(&db_path) {
        Ok(storage) => storage,
        Err(e) => {
            tracing::error!("Failed to create storage at {}: {}", db_path, e);
            return Err(e.into());
        }
    };

    let signing_keys = SigningKeys::new(3600 * 24 * 30, 3600); // 30 days key expire, 1 hour sig expire

    let state = AppState {
        keycloak_url,
        realm: std::env::var("REALM").map_err(|e| {
            tracing::error!("REALM not set: {}", e);
            e
        })?,
        client_id: std::env::var("CLIENT_ID").map_err(|e| {
            tracing::error!("CLIENT_ID not set: {}", e);
            e
        })?,
        client_secret: std::env::var("CLIENT_SECRET").map_err(|e| {
            tracing::error!("CLIENT_SECRET not set: {}", e);
            e
        })?,
        base_path: std::env::var("BASE_PATH").unwrap_or("/srv/media".to_string()),
        http_client: Client::new(),
        config: config.clone(),
        channel_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        storage: std::sync::Arc::new(std::sync::Mutex::new(storage)),
        signing_keys: std::sync::Arc::new(std::sync::Mutex::new(signing_keys)),
    };

    // Start file monitoring in background
    let watch_path = std::env::var("WATCH_PATH").unwrap_or("".to_string());
    let rss_outpath = std::env::var("RSS_OUT_PATH").unwrap_or("/srv/aux/rss".to_string());
    let file_pattern = std::env::var("FILE_PATTERN").unwrap_or(r"zsv[\d]{6}.*\.docx".to_string());
    let rss_days = std::env::var("RSS_DAYS").unwrap_or("-1".to_string()).parse::<i32>().ok();

    let monitor_config = webfs::webfs::file_monitor::MonitorConfig {
        config: config.clone(),
        db_path: db_path.clone(),
        video_descr_file_pattern: file_pattern.clone(),
        rss_days: rss_days.unwrap_or(7),
        rss_output_path: rss_outpath.clone(),
        video_list_path: watch_path.clone(),
    };
    tracing::info!("Starting rss outpath for path: {}", rss_outpath);
    tracing::info!("Starting file monitor for path: {} and file pattern: {}", watch_path, file_pattern);
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = webfs::webfs::file_monitor::start_file_monitor(&monitor_config, state_clone.storage, state_clone.channel_cache).await {
            tracing::error!("File monitor error: {}", e);
        }
    });

    let app = Router::new()
        .route("/auth/v1/login", post(authenticate_handler))
        .route("/auth/v1/refresh", post(refresh_handler))
        .route("/auth/v1/signurl", post(signurl_handler))
        .route("/auth/v1/nginx", get(nginx_handler))
        .route("/fs/v1/", get(list_files_root_handler))
        .route("/fs/v1/{*path}", get(list_files_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener_type = if let Ok(socket_path) = std::env::var("API_SOCKET") {
        if !socket_path.is_empty() {
            Some(socket_path)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(socket_path) = listener_type {
        serve_unix(app, socket_path).await?;
    } else {
        let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
        serve_tcp(app, port).await?;
    }

    Ok(())
}

async fn serve_tcp(app: Router, port: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Server running on http://0.0.0.0:{}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.map_err(|e| {
        tracing::error!("Failed to bind TcpListener on port {}: {}", port, e);
        e
    })?;
    tokio::select! {
        result = axum::serve(listener, app) => {
            result.map_err(|e| {
                tracing::error!("Failed to serve TCP: {}", e);
                e
            })?;
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Received SIGINT, shutting down TCP server");
        }
    }
    Ok(())
}

async fn serve_unix(app: Router, socket_path: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Server running on socket: {}", socket_path);
    // Remove existing socket file if it exists to avoid bind failure
    std::fs::remove_file(&socket_path).ok();
    let listener = UnixListener::bind(&socket_path).map_err(|e| {
        tracing::error!("Failed to bind UnixListener on {}: {}", socket_path, e);
        e
    })?;
    tokio::select! {
        result = axum::serve(listener, app) => {
            result.map_err(|e| {
                tracing::error!("Failed to serve Unix: {}", e);
                e
            })?;
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Received SIGINT, shutting down Unix server");
        }
    }
    Ok(())
}
