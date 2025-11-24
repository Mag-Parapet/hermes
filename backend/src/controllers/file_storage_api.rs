use std::sync::Arc;
use axum::{
    extract::{State, Json, Path, Multipart},
    http::StatusCode, 
    response::IntoResponse
};
use serde_json::json;
use sqlx::{Row};
use uuid::Uuid;

use crate::{
    config::AppState,
    middlewares::file_storage_api::FileStorageApiKeyAuth,
    models::file::FileItem,
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

pub async fn upload_file(
    State(data): State<Arc<AppState>>,
    FileStorageApiKeyAuth(storage): FileStorageApiKeyAuth,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
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
    FileStorageApiKeyAuth(storage): FileStorageApiKeyAuth,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let file = sqlx::query_as::<_, FileItem>("SELECT * FROM files WHERE id = $1 AND file_storage_id = $2")
        .bind(file_id)
        .bind(storage.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "File not found or access denied"}))))?;

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

    Ok(Json(json!({"status": "success", "message": "File deleted"})))
}