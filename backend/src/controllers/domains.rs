use std::sync::Arc;
use axum::{extract::{State, Json, Path, Query}, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::{QueryBuilder, Postgres};
use uuid::Uuid;

use crate::{
    config::AppState,
    middlewares::auth::JWTAuth,
    models::domain::Domain,
    dtos::{
        domains::{CreateDomainSchema, UpdateDomainSchema, DomainFilterOptions, PaginatedDomainList},
        pagination::PaginationMeta,
    },
    utils::{nginx::NginxManager, domain_validator::DomainValidator},
};


pub async fn create_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Json(body): Json<CreateDomainSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    // 1. Basic Domain Syntax Validation
    if let Err(e) = DomainValidator::validate_domain_name(&body.domain) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
    }

    let existing_entry = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE domain = $1")
        .bind(&body.domain)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if existing_entry.is_some() {
        return Err((StatusCode::CONFLICT, Json(json!({"message": "Domain already managed"}))));
    }

    let client_max_body_size = body.client_max_body_size.unwrap_or(data.env.multer_max_file_size as i64).max(0);
    
    let cert_path = body.ssl_certificate_path.unwrap_or_else(|| data.env.default_ssl_cert_path.clone());
    let key_path = body.ssl_certificate_key_path.unwrap_or_else(|| data.env.default_ssl_key_path.clone());
    let is_ssl = body.is_ssl.unwrap_or(data.env.default_ssl_enabled);

    let nginx_target_host = body.nginx_target_host.as_deref();
    let nginx_root_path = body.nginx_root_path.as_deref();
    let nginx_config_content = body.nginx_config_content.as_deref();
    
    let is_nginx_type = matches!(body.domain_type.as_str(), "reverse_proxy" | "web_server" | "static_host" | "custom");

    if is_nginx_type {
        match body.domain_type.as_str() {
            "reverse_proxy" => {
                if let Some(target) = nginx_target_host {
                    if let Err(e) = DomainValidator::validate_proxy_target(target) {
                        return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
                    }
                } else {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Type 'reverse_proxy' requires 'nginxTargetHost'"}))));
                }
            }
            "web_server" | "static_host" => {
                if let Some(root) = nginx_root_path {
                     if let Err(e) = DomainValidator::validate_root_path(root) {
                        return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
                    }
                } else {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Web/Static types require 'nginxRootPath'"}))));
                }
            }
            "custom" => {
                if nginx_config_content.is_none() {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Custom type requires 'nginxConfigContent'"}))));
                }
                if nginx_config_content.unwrap().trim().is_empty() {
                     return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Custom configuration cannot be empty"}))));
                }
            }
            _ => {}
        }

        if is_ssl {
            if let Err(e) = DomainValidator::validate_ssl_files(&cert_path, &key_path) {
                return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
            }
        }

    } else {
          return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Invalid or unsupported domain type specified"}))));
    }
    
    // 3. Attempt Nginx Deployment
    if is_nginx_type {
        match NginxManager::deploy_site(
            &body.domain_type,
            &body.domain, 
            nginx_target_host, 
            nginx_root_path,
            client_max_body_size, // Passing bytes
            is_ssl,
            &cert_path,
            &key_path,
            nginx_config_content
        ) {
            Ok(_) => { /* continue */ }
            Err(e) => {
                tracing::error!("Nginx deployment failed: {}", e);
                return Err((StatusCode::BAD_REQUEST, Json(json!({"status": "fail", "message": e}))));
            }
        }
    }
    
    // 4. Save to DB
    // Ensure DB schema expects bytes (INTEGER/BIGINT)
    let domain = sqlx::query_as::<_, Domain>(
        r#"
        INSERT INTO domains (
            user_id, domain, client_max_body_size, is_ssl, is_active, 
            domain_type, nginx_target_host, nginx_root_path, nginx_config_content,
            ssl_certificate_path, ssl_certificate_key_path
        ) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) 
        RETURNING *
        "#
    )
    .bind(user.id)
    .bind(&body.domain)
    .bind(client_max_body_size)
    .bind(is_ssl)
    .bind(true)
    .bind(&body.domain_type)
    .bind(body.nginx_target_host)
    .bind(body.nginx_root_path)
    .bind(body.nginx_config_content)
    .bind(cert_path)
    .bind(key_path)
    .fetch_one(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": domain}))))
}

pub async fn get_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Domain not found"}))))?;

    Ok(Json(json!({"status": "success", "data": domain})))
}

pub async fn update_domain(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDomainSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let existing = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Domain not found"}))))?;

    let new_domain_name = body.domain.as_deref().unwrap_or(&existing.domain);
    let new_domain_type = body.domain_type.as_deref().unwrap_or(&existing.domain_type);
    
    if body.domain.is_some() {
        if let Err(e) = DomainValidator::validate_domain_name(new_domain_name) {
             return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
        }
    }

    let new_target_host = body.nginx_target_host.as_deref().or(existing.nginx_target_host.as_deref());
    let new_root_path = body.nginx_root_path.as_deref().or(existing.nginx_root_path.as_deref());
    let new_config_content = body.nginx_config_content.as_deref().or(existing.nginx_config_content.as_deref());

    // CHANGED: Use bytes. Fallback to existing value which is already bytes.
    let new_max_body_size = body.client_max_body_size.or(existing.client_max_body_size).unwrap_or(10485760); // Default 10MB in bytes
    
    let new_is_ssl = body.is_ssl.or(existing.is_ssl).unwrap_or(false);
    let new_cert_path = body.ssl_certificate_path.as_deref().or(existing.ssl_certificate_path.as_deref()).unwrap_or("default");
    let new_key_path = body.ssl_certificate_key_path.as_deref().or(existing.ssl_certificate_key_path.as_deref()).unwrap_or("default");

    let is_nginx_type = matches!(new_domain_type, "reverse_proxy" | "web_server" | "static_host" | "custom");

    if is_nginx_type {
        match new_domain_type {
            "reverse_proxy" => {
                if let Some(target) = new_target_host {
                    if let Err(e) = DomainValidator::validate_proxy_target(target) {
                        return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
                    }
                } else {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Type 'reverse_proxy' requires 'nginxTargetHost'"}))));
                }
            }
            "web_server" | "static_host" => {
                 if let Some(root) = new_root_path {
                     if let Err(e) = DomainValidator::validate_root_path(root) {
                        return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
                    }
                } else {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Web/Static types require 'nginxRootPath'"}))));
                }
            }
            "custom" => {
                if new_config_content.is_none() {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Custom type requires 'nginxConfigContent'"}))));
                }
            }
            _ => {}
        }
        
        // if new_is_ssl {
        //     if let Err(e) = DomainValidator::validate_ssl_files(new_cert_path, new_key_path) {
        //         return Err((StatusCode::BAD_REQUEST, Json(json!({"message": e}))));
        //     }
        // }
    }

    let name_changed = new_domain_name != existing.domain;
    if name_changed {
        let _ = NginxManager::delete_site(&existing.domain);
    }

    if is_nginx_type {
        match NginxManager::deploy_site(
            new_domain_type,
            new_domain_name,
            new_target_host,
            new_root_path,
            new_max_body_size,
            new_is_ssl,
            new_cert_path,
            new_key_path,
            new_config_content
        ) {
            Ok(_) => {},
            Err(e) => {
                tracing::error!("Nginx update deployment failed: {}", e);
                return Err((StatusCode::BAD_REQUEST, Json(json!({"status": "fail", "message": e}))));
            }
        }
    }

    let updated_domain = sqlx::query_as::<_, Domain>(
        r#"
        UPDATE domains SET 
            domain = $1, client_max_body_size = $2, is_ssl = $3, is_active = $4,
            domain_type = $5, nginx_target_host = $6, nginx_root_path = $7, nginx_config_content = $8,
            ssl_certificate_path = $9, ssl_certificate_key_path = $10,
            updated_at = NOW()
        WHERE id = $11
        RETURNING *
        "#
    )
    .bind(new_domain_name)
    .bind(new_max_body_size)
    .bind(new_is_ssl)
    .bind(true)
    .bind(new_domain_type)
    .bind(new_target_host)
    .bind(new_root_path)
    .bind(new_config_content)
    .bind(new_cert_path)
    .bind(new_key_path)
    .bind(id)
    .fetch_one(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "data": updated_domain})))
}

pub async fn list_domains(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Query(opts): Query<DomainFilterOptions>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let page = opts.page.unwrap_or(1).max(1);
    let page_size = opts.page_size.unwrap_or(10).max(1);
    let offset = (page - 1) * page_size;

    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM domains WHERE user_id = ");
    count_qb.push_bind(user.id);
    
    if let Some(ref search) = opts.search {
        count_qb.push(" AND domain ILIKE ");
        count_qb.push_bind(format!("%{}%", search));
    }
    if let Some(ref d_type) = opts.domain_type {
        count_qb.push(" AND domain_type = ");
        count_qb.push_bind(d_type);
    }
    if let Some(active) = opts.is_active {
        count_qb.push(" AND is_active = ");
        count_qb.push_bind(active);
    }

    let total_items: i64 = count_qb.build_query_scalar().fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut query_qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM domains WHERE user_id = ");
    query_qb.push_bind(user.id);

    if let Some(ref search) = opts.search {
        query_qb.push(" AND domain ILIKE ");
        query_qb.push_bind(format!("%{}%", search));
    }
    if let Some(ref d_type) = opts.domain_type {
        query_qb.push(" AND domain_type = ");
        query_qb.push_bind(d_type);
    }
    if let Some(active) = opts.is_active {
        query_qb.push(" AND is_active = ");
        query_qb.push_bind(active);
    }

    query_qb.push(" ORDER BY created_at DESC LIMIT ");
    query_qb.push_bind(page_size);
    query_qb.push(" OFFSET ");
    query_qb.push_bind(offset);

    let domains: Vec<Domain> = query_qb.build_query_as()
        .fetch_all(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let total_pages = (total_items as f64 / page_size as f64).ceil() as i64;

    Ok(Json(json!({
        "status": "success", 
        "data": PaginatedDomainList {
            domains,
            pagination: PaginationMeta { total_pages, page_size, total_items }
        }
    })))
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

    let is_nginx = matches!(domain.domain_type.as_str(), "reverse_proxy" | "web_server" | "static_host" | "custom");
    if is_nginx {
        let _ = NginxManager::delete_site(&domain.domain);
    }
    
    sqlx::query("DELETE FROM domains WHERE id = $1")
        .bind(id)
        .execute(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "message": "Domain deleted"})))
}