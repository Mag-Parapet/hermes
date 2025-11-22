use std::sync::Arc;
use axum::{extract::{State, Json, Path}, http::StatusCode, response::IntoResponse};
use serde_json::json;
use crate::{
    config::AppState,
    middlewares::auth::JWTAuth,
    models::{dtos::CreateDomainSchema, domain::Domain},
    utils::nginx::NginxManager,
};

pub async fn create_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Json(body): Json<CreateDomainSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    // 1. Validation: Check if domain exists in DB
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM domains WHERE domain = $1)")
        .bind(&body.domain)
        .fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if exists {
        return Err((StatusCode::CONFLICT, Json(json!({"message": "Domain already managed"}))));
    }

    // 2. Attempt Nginx Deployment
    let max_mb = body.max_body_size.unwrap_or(50);
    
    // We try to deploy BEFORE saving to DB (or you can save as 'pending' then update)
    // Here we do it synchronously for simplicity.
    match NginxManager::deploy_site(&body.domain, body.port, max_mb) {
        Ok(_) => {
            let domain = sqlx::query_as::<_, Domain>(
                "INSERT INTO domains (user_id, domain, port, max_body_size, is_active) VALUES ($1, $2, $3, $4, $5) RETURNING *"
            )
            .bind(user.id)
            .bind(&body.domain)
            .bind(body.port)
            .bind(max_mb)
            .bind(true)
            .fetch_one(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

            Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": domain}))))
        }
        Err(e) => {
            tracing::error!("Nginx deployment failed: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(json!({"status": "fail", "message": e}))))
        }
    }
}

pub async fn list_domains(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let domains = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE user_id = $1 ORDER BY created_at DESC")
        .bind(user.id)
        .fetch_all(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "data": domains})))
}

pub async fn delete_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Domain not found"}))))?;

    let _ = NginxManager::delete_site(&domain.domain);

    sqlx::query("DELETE FROM domains WHERE id = $1")
        .bind(id)
        .execute(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "message": "Domain deleted"})))
}