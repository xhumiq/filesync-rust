use webfs::auth::handler::{authenticate_handler, refresh_handler};

use axum::{
    routing::post,
    Router,
};
use tokio::net::{TcpListener, UnixListener};
use webfs::AppState;
use reqwest::Client;
use tower_http::cors::CorsLayer;
use webfs::models::files::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();

    if let Some(timestamp) = option_env!("VERGEN_BUILD_TIMESTAMP") {
        println!("Build Timestamp: {timestamp}");
    }
    if let Some(describe) = option_env!("VERGEN_GIT_DESCRIBE") {
        println!("git describe: {describe}");
    }

    webfs::init_tracing("../logs/webfs.log")?;

    let config_path = std::env::var("CONFIG_PATH").unwrap_or("config-test.yaml".to_string());
    tracing::info!("Application started Config: {}", config_path);

    let config = Channel::read_config(&config_path)?;

    let state = AppState {
        keycloak_url: std::env::var("KEYCLOAK_URL")?,
        realm: std::env::var("REALM")?,
        client_id: std::env::var("CLIENT_ID")?,
        client_secret: std::env::var("CLIENT_SECRET")?,
        base_path: std::env::var("BASE_PATH").unwrap_or("/srv/media".to_string()),
        http_client: Client::new(),
        config: config.clone(),
        channel_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    };

    // Start file monitoring in background
    let db_path = std::env::var("DB_PATH").unwrap_or("/opt/webdav/data/webfs".to_string());
    let watch_path = std::env::var("WATCH_PATH").unwrap_or("/srv/media/Video".to_string());
    let rss_outpath = std::env::var("RSS_OUT_PATH").unwrap_or("/srv/rss".to_string());
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

    tracing::info!("Starting file monitor for path: {} and file pattern: {}", watch_path, file_pattern);
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = webfs::webfs::file_monitor::start_file_monitor(&monitor_config, state_clone.channel_cache).await {
            tracing::error!("File monitor error: {}", e);
        }
    });

    let app = Router::new()
        .route("/auth/v1/login", post(authenticate_handler))
        .route("/auth/v1/refresh", post(refresh_handler))
        .route("/fs/v1/{*path}", axum::routing::get(webfs::webfs::handler::list_files_handler))
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
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server running on http://0.0.0.0:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_unix(app: Router, socket_path: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = UnixListener::bind(&socket_path)?;
    tracing::info!("Server running on socket: {}", socket_path);
    axum::serve(listener, app).await?;
    Ok(())
}
