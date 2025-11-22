use axum::{routing::get, Router};
use std::sync::Arc;
use crate::{config::AppState, controllers::users};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/me", get(users::get_me))
}