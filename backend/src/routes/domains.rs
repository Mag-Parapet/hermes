use axum::{routing::{post, get, delete}, Router};
use std::sync::Arc;
use crate::{config::AppState, controllers::domains};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/domains", post(domains::create_domain))
        .route("/domains", get(domains::list_domains))
        .route("/domains/:id", delete(domains::delete_domain))
}