use std::sync::Arc;
use std::path::Path as StdPath;
use axum::{
    extract::{State, Json, Path, Query, Multipart},
    http::StatusCode, 
    response::IntoResponse
};
use serde_json::json;
use sqlx::{QueryBuilder, Postgres};
use uuid::Uuid;

use crate::{
    config::AppState,
    middlewares::auth::JWTAuth,
    models::{
        file_storage::FileStorage,
        file::FileItem,
    },
    dtos::{
        file_storage::{CreateFileStorageSchema, PaginatedFileStorageList, FileStorageFilterOptions, UpdateFileStorageSchema},
        file::{FileFilterOptions, PaginatedFileList, CreateFolderSchema},
        pagination::PaginationMeta,
    },
    utils::file_ops::FileOps,
};

fn map_file_to_response(file: FileItem, base_url: &str) -> crate::dtos::file::FileResponse {
    let base_path = format!("{}/{}", base_url, file.file_storage_id);
    let mut url = format!("{}/{}", base_path, file.name);
    let mut variants = None;

    if file.file_type.starts_with("image/") {
        let ext = file.file_type.strip_prefix("image/").unwrap_or("jpeg");
        url = format!("{}/{}.{}", base_path, file.id, ext);
        
        variants = Some(crate::dtos::file::ImageVariants {
            original: url.clone(),
            blurhash: file.blurhash.clone(),
            sm: format!("{}/{}_sm.{}", base_path, file.id, ext),
            md: format!("{}/{}_md.{}", base_path, file.id, ext),
            lg: format!("{}/{}_lg.{}", base_path, file.id, ext),
        });
    }

    crate::dtos::file::FileResponse {
        id: file.id,
        file_storage_id: file.file_storage_id,
        name: file.name,
        path: file.path,
        size: file.size,
        file_type: file.file_type,
        is_folder: file.is_folder,
        created_at: file.created_at,
        updated_at: file.updated_at,
        url,
        variants,
    }
}

pub async fn create_storage(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Json(body): Json<CreateFileStorageSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let quota_size = body.quota_size.unwrap_or(data.env.default_storage_quota);

    let storage = sqlx::query_as::<_, FileStorage>(
        r#"
        INSERT INTO file_storages 
        (user_id, name, file_max_size, compression_enabled, img_resize_max_size, img_default_format, allowed_file_types, is_active, quota_size, current_size) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0) 
        RETURNING *
        "#
    )
    .bind(user.id)
    .bind(body.name)
    .bind(body.file_max_size.unwrap_or(5242880))
    .bind(body.compression_enabled.unwrap_or(false))
    .bind(body.img_resize_max_size.unwrap_or(1920))
    .bind(body.img_default_format.unwrap_or("webp".to_string())) 
    .bind(body.allowed_file_types.unwrap_or("*".to_string()))
    .bind(body.is_active.unwrap_or(true))
    .bind(quota_size) 
    .fetch_one(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": storage}))))
}

pub async fn list_storages(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Query(opts): Query<FileStorageFilterOptions>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let page = opts.page.unwrap_or(1).max(1);
    let page_size = opts.page_size.unwrap_or(10).max(1);
    let offset = (page - 1) * page_size;

    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM file_storages WHERE user_id = ");
    count_qb.push_bind(user.id);
    if let Some(ref search) = opts.search {
        count_qb.push(" AND name ILIKE ");
        count_qb.push_bind(format!("%{}%", search));
    }
    let total_items: i64 = count_qb.build_query_scalar().fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM file_storages WHERE user_id = ");
    qb.push_bind(user.id);
    if let Some(ref search) = opts.search {
        qb.push(" AND name ILIKE ");
        qb.push_bind(format!("%{}%", search));
    }
    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(page_size);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let storages: Vec<FileStorage> = qb.build_query_as().fetch_all(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let total_pages = (total_items as f64 / page_size as f64).ceil() as i64;

    Ok(Json(json!({
        "status": "success",
        "data": PaginatedFileStorageList {
            file_storages: storages,
            pagination: PaginationMeta { total_pages, page_size, total_items }
        }
    })))
}

pub async fn get_storage(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(storage_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let storage = sqlx::query_as::<_, FileStorage>("SELECT * FROM file_storages WHERE id = $1 AND user_id = $2")
        .bind(storage_id)
        .bind(user.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Storage not found"}))))?;

    Ok(Json(json!({"status": "success", "data": storage})))
}

pub async fn update_storage(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(storage_id): Path<Uuid>,
    Json(body): Json<UpdateFileStorageSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let existing_storage = sqlx::query_as::<_, FileStorage>("SELECT * FROM file_storages WHERE id = $1 AND user_id = $2")
        .bind(storage_id)
        .bind(user.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Storage not found or access denied"}))))?;

    let new_name = body.name.as_deref().unwrap_or(&existing_storage.name);
    let new_file_max_size = body.file_max_size.unwrap_or(existing_storage.file_max_size);
    let new_compression_enabled = body.compression_enabled.unwrap_or(existing_storage.compression_enabled);
    let new_img_resize_max_size = body.img_resize_max_size.unwrap_or(existing_storage.img_resize_max_size);
    let new_img_default_format = body.img_default_format.as_deref().unwrap_or(&existing_storage.img_default_format);
    let new_allowed_file_types = body.allowed_file_types.as_deref().unwrap_or(&existing_storage.allowed_file_types);
    let new_is_active = body.is_active.unwrap_or(existing_storage.is_active);
    let new_quota_size = body.quota_size.unwrap_or(existing_storage.quota_size);

    let storage = sqlx::query_as::<_, FileStorage>(
        r#"
        UPDATE file_storages SET 
            name = $1,
            file_max_size = $2,
            compression_enabled = $3,
            img_resize_max_size = $4,
            img_default_format = $5,
            allowed_file_types = $6,
            is_active = $7,
            quota_size = $10,
            updated_at = NOW()
        WHERE id = $8 AND user_id = $9
        RETURNING *
        "#
    )
    .bind(new_name)
    .bind(new_file_max_size)
    .bind(new_compression_enabled)
    .bind(new_img_resize_max_size)
    .bind(new_img_default_format)
    .bind(new_allowed_file_types)
    .bind(new_is_active)
    .bind(storage_id)
    .bind(user.id)
    .bind(new_quota_size)
    .fetch_one(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "data": storage})))
}

pub async fn delete_storage(
    State(data): State<Arc<AppState>>,
    JWTAuth(user): JWTAuth,
    Path(storage_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let storage = sqlx::query_as::<_, FileStorage>("SELECT * FROM file_storages WHERE id = $1 AND user_id = $2")
        .bind(storage_id)
        .bind(user.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Storage not found or access denied"}))))?;

    let storage_path = format!("{}/{}", data.env.storage_root, storage.id);
    
    match FileOps::delete_folder(&storage_path) {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Failed to delete physical storage folder {}: {}", storage_path, e);
        }
    }

    sqlx::query("DELETE FROM file_storages WHERE id = $1")
        .bind(storage_id)
        .execute(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "message": "Storage bucket and all contents deleted"})))
}

pub async fn list_files(
    State(data): State<Arc<AppState>>,
    JWTAuth(_user): JWTAuth,
    Path(storage_id): Path<Uuid>,
    Query(opts): Query<FileFilterOptions>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let page = opts.page.unwrap_or(1).max(1);
    let page_size = opts.page_size.unwrap_or(100).max(1);
    let offset = (page - 1) * page_size;
    let target_path = opts.path.unwrap_or("/".to_string());
    
    let is_global_search = opts.search.is_some() && !opts.search.as_ref().unwrap().trim().is_empty();

    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM files WHERE file_storage_id = ");
    count_qb.push_bind(storage_id);

    if is_global_search {
        let search_term = opts.search.as_ref().unwrap();
        count_qb.push(" AND name ILIKE ");
        count_qb.push_bind(format!("%{}%", search_term));
    } else {
        count_qb.push(" AND path = ");
        count_qb.push_bind(&target_path);
    }

    let total_items: i64 = count_qb.build_query_scalar().fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM files WHERE file_storage_id = ");
    qb.push_bind(storage_id);

    if is_global_search {
        let search_term = opts.search.as_ref().unwrap();
        qb.push(" AND name ILIKE ");
        qb.push_bind(format!("%{}%", search_term));
    } else {
        qb.push(" AND path = ");
        qb.push_bind(&target_path);
    }
    
    qb.push(" ORDER BY is_folder DESC, name ASC LIMIT ");
    qb.push_bind(page_size);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let files: Vec<FileItem> = qb.build_query_as().fetch_all(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let responses: Vec<_> = files.into_iter()
        .map(|f| map_file_to_response(f, &data.env.static_host))
        .collect();

    let total_pages = (total_items as f64 / page_size as f64).ceil() as i64;

    Ok(Json(json!({
        "status": "success",
        "data": PaginatedFileList {
            files: responses,
            pagination: PaginationMeta { total_pages, page_size, total_items },
            current_path: if is_global_search { "Global Search".to_string() } else { target_path }
        }
    })))
}

pub async fn create_folder(
    State(data): State<Arc<AppState>>,
    JWTAuth(_user): JWTAuth,
    Path(storage_id): Path<Uuid>,
    Json(body): Json<CreateFolderSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let storage_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM file_storages WHERE id = $1)")
        .bind(storage_id)
        .fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if !storage_exists {
        return Err((StatusCode::NOT_FOUND, Json(json!({"message": "Storage bucket not found"}))));
    }

    // Safety: Truncate folder name if strict DB limit exists
    let safe_name: String = body.name.chars().take(50).collect();

    let folder = sqlx::query_as::<_, FileItem>(
        "INSERT INTO files (file_storage_id, name, path, size, file_type, is_folder) VALUES ($1, $2, $3, 0, 'folder', true) RETURNING *"
    )
    .bind(storage_id)
    .bind(safe_name)
    .bind(body.path) 
    .fetch_one(&data.db).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": map_file_to_response(folder, &data.env.static_host)}))))
}

pub async fn upload_file(
    State(data): State<Arc<AppState>>,
    JWTAuth(_user): JWTAuth,
    Path(storage_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let storage = sqlx::query_as::<_, FileStorage>("SELECT * FROM file_storages WHERE id = $1")
        .bind(storage_id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "Storage not found"}))))?;

    if !storage.is_active {
        return Err((StatusCode::FORBIDDEN, Json(json!({"message": "Storage bucket is currently inactive"}))));
    }

    let mut uploaded_files = Vec::new();
    let mut current_path = "/".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))))? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "path" {
            if let Ok(txt) = field.text().await {
                // VALIDATION 1: Path Traversal & format
                if txt.contains("..") {
                    return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Invalid path: Directory traversal not allowed"}))));
                }
                // Ensure absolute path format
                current_path = if txt.starts_with('/') { txt } else { format!("/{}", txt) };
            }
            continue;
        }

        if let Some(raw_filename) = field.file_name() {
             // VALIDATION 2: Sanitize Filename (Strip directory components)
            let filename = StdPath::new(raw_filename)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown_file")
                .to_string();

            // Capture MIME type from multipart header
            let mime_type = field.content_type().map(|s| s.to_string()).unwrap_or("application/octet-stream".to_string());
            
            let data_bytes = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
            
             // VALIDATION 3: Empty File Check
            if data_bytes.is_empty() {
                return Err((StatusCode::BAD_REQUEST, Json(json!({"message": "Cannot upload empty file"}))));
            }

            let file_size_estimate = data_bytes.len() as i64;
            
            if file_size_estimate > storage.file_max_size {
                return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(json!({"message": format!("File {} exceeds max file size of {} bytes", filename, storage.file_max_size)}))));
            }
            
            if storage.current_size + file_size_estimate > storage.quota_size {
                 return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(json!({"message": format!("File {} upload would exceed the storage quota of {} bytes", filename, storage.quota_size)}))));
            }

            if !FileOps::is_allowed_type(&filename, &storage.allowed_file_types) {
                 return Err((StatusCode::BAD_REQUEST, Json(json!({"message": format!("File type of {} not allowed", filename)}))));
            }

            let file_id = Uuid::new_v4();

            // Safety: Truncate filename if too long for DB (assuming 50 char limit based on error)
            let safe_filename = if filename.len() > 50 {
                let path = StdPath::new(&filename);
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&filename);
                
                let available_len = 50usize.saturating_sub(ext.len()).saturating_sub(1); // -1 for dot
                if available_len > 0 {
                    format!("{}.{}", stem.chars().take(available_len).collect::<String>(), ext)
                } else {
                    filename.chars().take(50).collect()
                }
            } else {
                filename.clone()
            };

            // Spawn blocking logic (kept from previous fix)
            let storage_root = data.env.storage_root.clone();
            let storage_id_clone = storage.id;
            let filename_clone = filename.clone(); // Use original for disk logic
            let data_vec = data_bytes.to_vec();
            let compression = storage.compression_enabled;
            let resize = storage.img_resize_max_size;
            let fmt = storage.img_default_format.clone();

            let (final_size, file_type, blurhash_opt) = tokio::task::spawn_blocking(move || {
                FileOps::save_file(
                    storage_root,
                    storage_id_clone,
                    file_id,
                    filename_clone,
                    data_vec,
                    compression,
                    resize,
                    fmt
                )
            })
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Thread join error: {}", e)}))))?
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

            // OVERRIDE "raw" with real MIME type from multipart
            let mut db_file_type = if file_type == "raw" { mime_type } else { file_type };
            
            // Safety: Fallback if mime type is too long for legacy DB column
            if db_file_type.len() > 50 {
                db_file_type = "application/octet-stream".to_string();
            }

            let file_record = sqlx::query_as::<_, FileItem>(
                r#"
                INSERT INTO files 
                (id, file_storage_id, name, path, size, file_type, blurhash) 
                VALUES ($1, $2, $3, $4, $5, $6, $7) 
                RETURNING *
                "#
            )
            .bind(file_id)
            .bind(storage.id)
            .bind(&safe_filename) // Use truncated name for DB
            .bind(&current_path)
            .bind(final_size)
            .bind(db_file_type) // Use sanitized mime type
            .bind(blurhash_opt)
            .fetch_one(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

            sqlx::query("UPDATE file_storages SET current_size = current_size + $1 WHERE id = $2")
                .bind(final_size)
                .bind(storage_id)
                .execute(&data.db).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to update storage size: {}", e.to_string())}))))?;

            uploaded_files.push(file_record);
        }
    }

    let responses: Vec<_> = uploaded_files.into_iter()
        .map(|f| map_file_to_response(f, &data.env.static_host))
        .collect();
        
    Ok((StatusCode::CREATED, Json(json!({"status": "success", "data": responses}))))
}

pub async fn delete_file(
    State(data): State<Arc<AppState>>,
    JWTAuth(_user): JWTAuth,
    Path((_storage_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    // 1. Fetch Target Item
    let target = sqlx::query_as::<_, FileItem>("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "File not found"}))))?;

    let target_storage_id = target.file_storage_id;
    let is_folder = target.is_folder;
    
    let folder_full_path = if is_folder {
        Some(if target.path == "/" {
            target.name.clone()
        } else {
            format!("{}/{}", target.path.trim_end_matches('/'), target.name)
        })
    } else {
        None
    };

    let mut files_to_delete = vec![target];
    
    if is_folder {
        if let Some(folder_path) = folder_full_path {
             let descendants = sqlx::query_as::<_, FileItem>(
                "SELECT * FROM files WHERE file_storage_id = $1 AND (path = $2 OR path LIKE $3)"
            )
            .bind(target_storage_id)
            .bind(&folder_path)
            .bind(format!("{}/%", folder_path))
            .fetch_all(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

            files_to_delete.extend(descendants);
        }
    }

    let mut ids_to_delete = Vec::new();
    let mut total_size_freed: i64 = 0;

    for file in &files_to_delete {
        ids_to_delete.push(file.id);
        
        if !file.is_folder {
            let physical_path = FileOps::get_physical_path(
                &data.env.storage_root, 
                file.file_storage_id, 
                file.id, 
                &file.file_type, 
                &file.name
            );
            if let Err(e) = FileOps::delete_file(&physical_path) {
                tracing::warn!("Failed to delete physical file {}: {}", file.id, e);
            }
            total_size_freed += file.size;
        }
    }

    sqlx::query("DELETE FROM files WHERE id = ANY($1)")
        .bind(&ids_to_delete)
        .execute(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if total_size_freed > 0 {
        sqlx::query("UPDATE file_storages SET current_size = current_size - $1 WHERE id = $2")
            .bind(total_size_freed)
            .bind(target_storage_id)  
            .execute(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to subtract size from storage: {}", e.to_string())}))))?;
    }

    Ok(Json(json!({"status": "success", "message": format!("Deleted {} item(s)", ids_to_delete.len())})))
}