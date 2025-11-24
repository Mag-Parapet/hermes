use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use crate::{
    config::AppState,
    models::file_storage::FileStorage,
};

// Renamed struct as requested to match file/purpose
pub struct FileStorageApiKeyAuth(pub FileStorage);

#[async_trait]
impl<S> FromRequestParts<S> for FileStorageApiKeyAuth
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let api_key_str = parts
            .headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"message": "Missing x-api-key Header"}))))?;

        let api_key = uuid::Uuid::parse_str(api_key_str)
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({"message": "Invalid API Key format"}))))?;

        let storage = sqlx::query_as::<_, FileStorage>("SELECT * FROM file_storages WHERE api_key = $1 AND is_active = true")
            .bind(api_key)
            .fetch_optional(&app_state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
            .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"message": "Invalid API Key"}))))?;

        Ok(FileStorageApiKeyAuth(storage))
    }
}