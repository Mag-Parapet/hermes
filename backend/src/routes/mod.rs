pub mod auth;
pub mod users;
pub mod domains;

use axum::Router;
use std::sync::Arc;
use crate::config::AppState;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .merge(auth::router())
        .merge(users::router())
        .merge(domains::router())
        .with_state(app_state)
}