pub mod auth;
pub mod users;
pub mod domains;
pub mod file_storage_api;
pub mod file_storage;

use axum::{
    Router,
    extract::DefaultBodyLimit,
};
use std::sync::Arc;
use crate::config::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .merge(auth::router())
        .merge(file_storage_api::router())
        .merge(file_storage::router())
        .merge(users::router())
        .merge(domains::router())
        .with_state(app_state)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))  // 1 GB limit
}