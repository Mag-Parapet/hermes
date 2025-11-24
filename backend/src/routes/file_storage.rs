use axum::{routing::{post, get, delete}, Router};
use std::sync::Arc;
use crate::{config::AppState, controllers::file_storage};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/storages", post(file_storage::create_storage))
        .route("/storages", get(file_storage::list_storages))
        .route("/storages/:storage_id/upload", post(file_storage::upload_file))
        .route("/storages/:storage_id/files", get(file_storage::list_files))
        .route("/storages/:storage_id/folders", post(file_storage::create_folder))
        .route("/storages/:storage_id/files/:file_id", delete(file_storage::delete_file))
}