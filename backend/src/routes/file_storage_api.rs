use axum::{routing::{post, delete}, Router};
use std::sync::Arc;
use crate::{config::AppState, controllers::file_storage_api};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/upload", post(file_storage_api::upload_file))
        .route("/api/files/:file_id", delete(file_storage_api::delete_file))
}