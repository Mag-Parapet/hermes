use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, StatusCode},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use crate::{
    config::AppState,
    models::user::User,
    utils::jwt::verify_token,
};

pub struct JWTAuth(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for JWTAuth
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"message": "Missing Authorization Header"}))))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"message": "Invalid Authorization Header"}))))?;

        let claims = verify_token(token, &app_state.env.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({"message": "Invalid Token"}))))?;

        let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap();
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&app_state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
            .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"message": "User not found"}))))?;

        Ok(JWTAuth(user))
    }
}