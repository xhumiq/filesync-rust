use webfs::auth::handler::{authenticate_handler, refresh_handler};

use axum::{
    routing::post,
    Router,
};
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
    };

    // Start file monitoring in background
    let db_path = std::env::var("DB_PATH")?;
    let watch_path = std::env::var("WATCH_PATH").unwrap_or("/srv/media/Video".to_string());
    let file_pattern = std::env::var("FILE_PATTERN").unwrap_or("^zsv.+\\.docx$".to_string());

    // tokio::spawn(async move {
    //     if let Err(e) = webfs::webfs::file_monitor::start_file_monitor(&db_path, &config.clone(), &file_pattern).await {
    //         tracing::error!("File monitor error: {}", e);
    //     }
    // });

    let app = Router::new()
        .route("/auth/v1/login", post(authenticate_handler))
        .route("/auth/v1/refresh", post(refresh_handler))
        .route("/fs/v1/*path", axum::routing::get(webfs::webfs::handler::list_files_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
