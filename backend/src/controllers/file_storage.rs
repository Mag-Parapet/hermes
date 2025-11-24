use std::sync::Arc;
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
        file_storage::{CreateFileStorageSchema, PaginatedFileStorageList, FileStorageFilterOptions},
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
    
    let storage = sqlx::query_as::<_, FileStorage>(
        r#"
        INSERT INTO file_storages 
        (user_id, name, file_max_size, compression_enabled, img_resize_max_size, img_default_format, allowed_file_types, is_active) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
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

    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM files WHERE file_storage_id = ");
    count_qb.push_bind(storage_id);
    count_qb.push(" AND path = ");
    count_qb.push_bind(&target_path);

    if let Some(ref search) = opts.search {
        count_qb.push(" AND name ILIKE ");
        count_qb.push_bind(format!("%{}%", search));
    }

    let total_items: i64 = count_qb.build_query_scalar().fetch_one(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM files WHERE file_storage_id = ");
    qb.push_bind(storage_id);
    qb.push(" AND path = ");
    qb.push_bind(&target_path);

    if let Some(ref search) = opts.search {
        qb.push(" AND name ILIKE ");
        qb.push_bind(format!("%{}%", search));
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
            current_path: target_path
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

    let folder = sqlx::query_as::<_, FileItem>(
        "INSERT INTO files (file_storage_id, name, path, size, file_type, is_folder) VALUES ($1, $2, $3, 0, 'folder', true) RETURNING *"
    )
    .bind(storage_id)
    .bind(body.name)
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

    let mut uploaded_files = Vec::new();
    let mut current_path = "/".to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))))? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "path" {
            if let Ok(txt) = field.text().await {
                current_path = txt;
            }
            continue;
        }

        if let Some(filename) = field.file_name() {
            let filename = filename.to_string();
            let data_bytes = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

            if data_bytes.len() as i64 > storage.file_max_size {
                return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(json!({"message": format!("File {} exceeds max size", filename)}))));
            }
            if !FileOps::is_allowed_type(&filename, &storage.allowed_file_types) {
                 return Err((StatusCode::BAD_REQUEST, Json(json!({"message": format!("File type of {} not allowed", filename)}))));
            }

            let file_id = Uuid::new_v4();

            let (final_size, file_type, blurhash_opt) = FileOps::save_file(
                &data.env.storage_root,
                storage.id,
                file_id,
                &filename,
                &data_bytes,
                storage.compression_enabled,
                storage.img_resize_max_size,
                &storage.img_default_format
            ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

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
            .bind(&filename)
            .bind(&current_path)
            .bind(final_size)
            .bind(file_type)
            .bind(blurhash_opt)
            .fetch_one(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

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

    let file = sqlx::query_as::<_, FileItem>("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "File not found"}))))?;

    if !file.is_folder {
        let physical_path = FileOps::get_physical_path(
            &data.env.storage_root, 
            file.file_storage_id, 
            file.id, 
            &file.file_type, 
            &file.name
        );
        FileOps::delete_file(&physical_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;
    }

    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"status": "success", "message": "Item deleted"})))
}