use axum::{routing::{post, get, put, delete}, Router};
use std::sync::Arc;
use crate::{config::AppState, controllers::domains};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/domains", post(domains::create_domain))
        .route("/domains", get(domains::list_domains))
        .route("/domains/:id", get(domains::get_domain))
        .route("/domains/:id", put(domains::update_domain))
        .route("/domains/:id", delete(domains::delete_domain))
}