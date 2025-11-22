use axum::{routing::post, Router};
use std::sync::Arc;
use crate::{config::AppState, controllers::auth};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
}