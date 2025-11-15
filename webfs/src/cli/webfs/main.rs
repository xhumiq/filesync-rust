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
async fn main() {
    dotenv::dotenv().ok();

    webfs::init_tracing("../logs/webfs.log");

    let config_path = std::env::var("CONFIG_PATH").unwrap_or("config-test.yaml".to_string());
    tracing::info!("Application started Config: {}", config_path);

    let config = Channel::read_config(&config_path).expect("Failed to load config");

    let state = AppState {
        keycloak_url: std::env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set"),
        realm: std::env::var("REALM").expect("REALM must be set"),
        client_id: std::env::var("CLIENT_ID").expect("CLIENT_ID must be set"),
        client_secret: std::env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set"),
        base_path: std::env::var("BASE_PATH").unwrap_or("/srv/media".to_string()),
        http_client: Client::new(),
        config: config.clone(),
    };

    // Start file monitoring in background
    let db_path = std::env::var("DB_PATH").expect("DB_PATH must be set");
    let watch_path = std::env::var("WATCH_PATH").unwrap_or("/srv/media/Video".to_string());
    let file_pattern = std::env::var("FILE_PATTERN").unwrap_or("^zsv.+\\.docx$".to_string());

    // tokio::spawn(async move {
    //     if let Err(e) = webfs::webfs::file_monitor::start_file_monitor(&db_path, &config.clone(), &file_pattern).await {
    //         tracing::error!("File monitor error: {}", e);
    //     }
    // });

    let app = Router::new()
        .route("/auth/login", post(authenticate_handler))
        .route("/auth/refresh", post(refresh_handler))
        .route("/files/*path", axum::routing::get(webfs::webfs::handler::list_files_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
