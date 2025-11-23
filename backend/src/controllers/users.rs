use axum::{response::IntoResponse, Json};
use serde_json::json;
use crate::{middlewares::auth::JWTAuth, dtos::users::UserResponse};

pub async fn get_me(JWTAuth(user): JWTAuth) -> impl IntoResponse {
    let response_user = UserResponse {
        id: user.id, 
        name: user.name, 
        email: user.email, 
        role: user.role, 
        created_at: user.created_at,
        last_login: user.last_login
    };

    Json(json!({"status": "success", "data": { "user": response_user }}))
}