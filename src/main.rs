use axum::{
    routing::{get, post},
    Router,
};
use axum::extract::DefaultBodyLimit;
use std::sync::Arc;
use reqwest::Client;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use iron_mask_proxy::{config, handlers};

#[tokio::main]
async fn main() {
    // 1. Initialize Logging
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // 2. Load Config
    let config = match config::AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            eprintln!("💡 Please ensure config.yaml exists and is valid");
            std::process::exit(1);
        }
    };
    
    let port = config.server.port;
    let host = config.server.host.clone();

    // 3. Setup Shared State
    let state = Arc::new(handlers::AppState {
        http_client: Client::builder()
            .timeout(Duration::from_millis(config.target.timeout_ms))
            .build()
            .expect("Failed to create HTTP client"),
        config: config.clone(),
    });

    // 4. Setup Routes & Layers
    let app = Router::new()
        .route("/mask", post(handlers::handle_log))
        .route("/healthz", get(handlers::health_check))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024)) // 2MB limit
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // 5. Start Server
    let addr = format!("{}:{}", host, port);
    
    println!("🚀 Iron Mask Proxy Professional Edition");
    println!("📡 Listening on: http://{}", addr);
    println!("💓 Health Check: http://{}/healthz", addr);
    
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("❌ Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    
    // Graceful Shutdown implementation
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("\n🛑 Signal received, starting graceful shutdown...");
}
