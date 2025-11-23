use std::sync::Arc;
use axum::{
    extract::{State, Json, Path, Query},
    http::StatusCode, 
    response::IntoResponse
};
use serde_json::json;
use sqlx::{QueryBuilder, Postgres};

use crate::{
    config::AppState, 
    dtos::{domains::{CreateDomainSchema, DomainFilterOptions, PaginatedDomainList, UpdateDomainSchema}, pagination::PaginationMeta}, 
    middlewares::auth::JWTAuth, 
    models::domain::Domain, 
    utils::nginx::NginxManager
};

pub async fn create_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Json(body): Json<CreateDomainSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    // 1. Validation: Check if domain OR port exists in DB
    let existing_entry = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE domain = $1 OR port = $2")
        .bind(&body.domain)
        .bind(body.port)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if let Some(entry) = existing_entry {
        if entry.domain == body.domain {
            return Err((StatusCode::CONFLICT, Json(json!({"message": "Domain already managed"}))));
        }
        if entry.port == body.port {
            return Err((StatusCode::CONFLICT, Json(json!({"message": format!("Port {} is already in use", body.port)}))));
        }
    }

    // 2. Prepare values
    let max_mb = body.max_body_size.unwrap_or(50);
    let is_ssl = body.is_ssl.unwrap_or(true);
    let is_active = body.is_active.unwrap_or(true);

    // 3. Attempt Nginx Deployment if active
    if is_active {
        let ssl_cert = body.ssl_certificate_path.as_deref();
        let ssl_key = body.ssl_certificate_key_path.as_deref();
        
        match NginxManager::deploy_site(&body.domain, body.port, max_mb, is_ssl, ssl_cert, ssl_key) {
            Ok(_) => {
                // 4. Success - Save to DB
                let domain = sqlx::query_as::<_, Domain>(
                    "INSERT INTO domains (user_id, domain, port, max_body_size, is_ssl, is_active, ssl_certificate_path, ssl_certificate_key_path) 
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
                     RETURNING *"
                )
                .bind(user.id)
                .bind(&body.domain)
                .bind(body.port)
                .bind(max_mb)
                .bind(is_ssl)
                .bind(is_active)
                .bind(&body.ssl_certificate_path)
                .bind(&body.ssl_certificate_key_path)
                .fetch_one(&data.db).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

                Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": domain}))))
            }
            Err(e) => {
                tracing::error!("Nginx deployment failed: {}", e);
                Err((StatusCode::BAD_REQUEST, Json(json!({"status": "fail", "message": e}))))
            }
        }
    } else {
        // Inactive - just save to DB
        let domain = sqlx::query_as::<_, Domain>(
            "INSERT INTO domains (user_id, domain, port, max_body_size, is_ssl, is_active, ssl_certificate_path, ssl_certificate_key_path) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING *"
        )
        .bind(user.id)
        .bind(&body.domain)
        .bind(body.port)
        .bind(max_mb)
        .bind(is_ssl)
        .bind(false)
        .bind(&body.ssl_certificate_path)
        .bind(&body.ssl_certificate_key_path)
        .fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": domain}))))
    }
}

pub async fn update_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<UpdateDomainSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    // 1. Fetch existing domain
    let existing_domain = sqlx::query_as::<_, Domain>(
        "SELECT * FROM domains WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Domain not found"}))))?;

    // 2. Check for conflicts if domain or port is being changed
    if body.domain != existing_domain.domain {
        let conflict = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE domain = $1 AND id != $2"
        )
        .bind(&body.domain)
        .bind(id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        if conflict.is_some() {
            return Err((StatusCode::CONFLICT, Json(json!({"message": "Domain already managed"}))));
        }
    }

    if body.port != existing_domain.port {
        let conflict = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE port = $1 AND id != $2"
        )
        .bind(body.port)
        .bind(id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        if conflict.is_some() {
            return Err((StatusCode::CONFLICT, Json(json!({"message": format!("Port {} is already in use", body.port)}))));
        }
    }

    // 3. Prepare updated values
    let updated_max_body = body.max_body_size.unwrap_or(
        existing_domain.max_body_size.unwrap_or(50)
    );
    let updated_ssl = body.is_ssl.unwrap_or(
        existing_domain.is_ssl.unwrap_or(true)
    );
    let updated_active = body.is_active.unwrap_or(
        existing_domain.is_active.unwrap_or(true)
    );
    
    // 4. Detect changes that require Nginx update
    let domain_changed = body.domain != existing_domain.domain;
    let port_changed = body.port != existing_domain.port;
    let max_body_changed = body.max_body_size.map_or(false, |new_val| {
        existing_domain.max_body_size.map_or(true, |old_val| new_val != old_val)
    });
    let ssl_changed = body.is_ssl.map_or(false, |new_val| {
        existing_domain.is_ssl.map_or(true, |old_val| new_val != old_val)
    });
    let activation_changed = body.is_active.map_or(false, |new_val| {
        existing_domain.is_active.map_or(true, |old_val| new_val != old_val)
    });
    let ssl_cert_changed = body.ssl_certificate_path.is_some() 
        && body.ssl_certificate_path.as_ref() != existing_domain.ssl_certificate_path.as_ref();
    let ssl_key_changed = body.ssl_certificate_key_path.is_some() 
        && body.ssl_certificate_key_path.as_ref() != existing_domain.ssl_certificate_key_path.as_ref();
    
    // Use new SSL paths if provided, otherwise keep existing ones
    let updated_ssl_cert = body.ssl_certificate_path.clone().or(existing_domain.ssl_certificate_path.clone());
    let updated_ssl_key = body.ssl_certificate_key_path.clone().or(existing_domain.ssl_certificate_key_path.clone());

    let needs_nginx_update = domain_changed || port_changed || max_body_changed 
        || ssl_changed || activation_changed || ssl_cert_changed || ssl_key_changed;

    // 5. Handle Nginx configuration changes
    if needs_nginx_update {
        // Remove old config if it was active
        if existing_domain.is_active.unwrap_or(false) {
            match NginxManager::delete_site(&existing_domain.domain) {
                Ok(_) => tracing::info!("Removed old Nginx config for: {}", existing_domain.domain),
                Err(e) => tracing::warn!("Failed to remove old Nginx config: {}", e),
            }
        }

        // Deploy new config if now active
        if updated_active {
            let ssl_cert = updated_ssl_cert.as_deref();
            let ssl_key = updated_ssl_key.as_deref();
            
            match NginxManager::deploy_site(&body.domain, body.port, updated_max_body, updated_ssl, ssl_cert, ssl_key) {
                Ok(_) => {
                    tracing::info!("Nginx configuration updated for domain: {}", body.domain);
                }
                Err(e) => {
                    tracing::error!("Nginx deployment failed: {}", e);
                    return Err((StatusCode::BAD_REQUEST, Json(json!({
                        "status": "fail", 
                        "message": format!("Failed to deploy Nginx configuration: {}", e)
                    }))));
                }
            }
        }
    }

    // 6. Update database
    let updated = sqlx::query_as::<_, Domain>(
        "UPDATE domains 
         SET domain = $1, port = $2, max_body_size = $3, is_ssl = $4, is_active = $5, 
             ssl_certificate_path = $6, ssl_certificate_key_path = $7, updated_at = NOW()
         WHERE id = $8 AND user_id = $9
         RETURNING *"
    )
    .bind(&body.domain)
    .bind(body.port)
    .bind(updated_max_body)
    .bind(updated_ssl)
    .bind(updated_active)
    .bind(&updated_ssl_cert)
    .bind(&updated_ssl_key)
    .bind(id)
    .bind(user.id)
    .fetch_one(&data.db).await
    .map_err(|e| {
        tracing::error!("Database update failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?;

    Ok(Json(json!({"status": "success", "data": updated})))
}

pub async fn list_domains(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Query(opts): Query<DomainFilterOptions>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let page = opts.page.unwrap_or(1).max(1);
    let page_size = opts.page_size.unwrap_or(10).max(1);
    let offset = (page - 1) * page_size;

    // Count query
    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM domains WHERE user_id = ");
    count_qb.push_bind(user.id);

    if let Some(ref search) = opts.search {
        if !search.is_empty() {
            count_qb.push(" AND domain ILIKE ");
            count_qb.push_bind(format!("%{}%", search));
        }
    }

    if let Some(port) = opts.port {
        count_qb.push(" AND port = ");
        count_qb.push_bind(port);
    }

    let total_items: i64 = count_qb.build_query_scalar()
        .fetch_one(&data.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    // Data query
    let mut query_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM domains WHERE user_id = ");
    query_qb.push_bind(user.id);

    if let Some(ref search) = opts.search {
        if !search.is_empty() {
            query_qb.push(" AND domain ILIKE ");
            query_qb.push_bind(format!("%{}%", search));
        }
    }

    if let Some(port) = opts.port {
        query_qb.push(" AND port = ");
        query_qb.push_bind(port);
    }

    query_qb.push(" ORDER BY created_at DESC LIMIT ");
    query_qb.push_bind(page_size);
    query_qb.push(" OFFSET ");
    query_qb.push_bind(offset);

    let domains: Vec<Domain> = query_qb.build_query_as()
        .fetch_all(&data.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let total_pages = (total_items as f64 / page_size as f64).ceil() as i64;
    
    let result = PaginatedDomainList {
        domains,
        pagination: PaginationMeta {
            total_pages,
            page_size,
            total_items,
        }
    };

    Ok(Json(json!({"status": "success", "data": result})))
}

pub async fn get_domain(
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

    Ok(Json(json!({"status": "success", "data": domain})))
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