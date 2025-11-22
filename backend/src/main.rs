mod config;
mod controllers;
mod middlewares;
mod models;
mod routes;
mod utils;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::routing::get;

use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

use crate::config::{Config, AppState, connect_db};
use crate::routes::create_router;

#[tokio::main]
async fn main() {
    let config = Config::init();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = connect_db().await;
    tracing::info!("✅ Connection to the database is successful!");

    let app_state = Arc::new(AppState {
        db: pool,
        env: config.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any) 
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = create_router(app_state)
        .route("/health", get(|| async { "Rust Auth Service is running!" }))
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    tracing::info!("🚀 Server started successfully on port {}", config.port);
    axum::serve(listener, app).await.unwrap();
}