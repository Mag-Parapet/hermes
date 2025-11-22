use std::sync::Arc;
use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::{
    config::AppState,
    models::{dtos::{RegisterUserSchema, LoginUserSchema, RefreshRequest, UserResponse, TokenResponse}, user::User},
    utils::{hash::hash_password, hash::verify_password, jwt::generate_tokens, jwt::verify_token},
};

pub async fn register(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let user_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(body.email.to_lowercase())
        .fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if user_exists {
        return Err((StatusCode::CONFLICT, Json(json!({"status": "fail", "message": "User exists"}))));
    }

    let hashed_password = hash_password(body.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, password, role) VALUES ($1, $2, $3, 'user') RETURNING *"
    )
    .bind(body.name)
    .bind(body.email.to_lowercase())
    .bind(hashed_password)
    .fetch_one(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    // Note: last_login is None on register, which is correct.

    let response_user = UserResponse {
        id: user.id, name: user.name, email: user.email, role: user.role, created_at: user.created_at, last_login: user.last_login
    };

    Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": { "user": response_user }}))))
}

pub async fn login(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(body.email.to_lowercase())
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"status": "fail", "message": "Invalid email/pass"}))))?;

    if !verify_password(&body.password, &user.password).unwrap_or(false) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"status": "fail", "message": "Invalid email/pass"}))));
    }

    // --- NEW: Update Last Login ---
    // We fire this query to update the timestamp. 
    // usage of 'let _ =' means we wait for it to finish, but don't care about the return result (just if it errors)
    let _ = sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&data.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    // ------------------------------

    let (access_token, refresh_token) = generate_tokens(user.id, user.role, &data.env.jwt_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    let token_res = TokenResponse {
        access_token,
        refresh_token,
    };

    Ok(Json(json!({"status": "success", "data": token_res})))
}

pub async fn refresh(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let claims = verify_token(&body.refresh_token, &data.env.jwt_secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({"message": "Invalid refresh token"}))))?;

    let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap();
    
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"message": "User not found"}))))?;

    let (access_token, refresh_token) = generate_tokens(user.id, user.role, &data.env.jwt_secret)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    let token_res = TokenResponse {
        access_token,
        refresh_token,
    };

    Ok(Json(json!({"status": "success", "data": token_res})))
}