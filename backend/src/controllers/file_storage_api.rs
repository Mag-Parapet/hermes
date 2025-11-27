use std::sync::Arc;
use std::path::Path as StdPath;
use axum::{
    extract::{State, Json, Path, Multipart},
    http::StatusCode, 
    response::IntoResponse
};
use serde_json::json;
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

        // Allow setting path via form field
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
            // Transforms "folder/file.txt" or "../../file.txt" -> "file.txt"
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

            // 1. Max File Size Check
            if file_size_estimate > storage.file_max_size {
                return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(json!({"message": format!("File {} exceeds max file size of {} bytes", filename, storage.file_max_size)}))));
            }

            // 2. Quota Check
            if storage.current_size + file_size_estimate > storage.quota_size {
                return Err((StatusCode::PAYLOAD_TOO_LARGE, Json(json!({"message": format!("File {} upload would exceed the storage quota of {} bytes", filename, storage.quota_size)}))));
            }

            // 3. File Type Check
            if !FileOps::is_allowed_type(&filename, &storage.allowed_file_types) {
                 return Err((StatusCode::BAD_REQUEST, Json(json!({"message": format!("File type of {} not allowed", filename)}))));
            }

            let file_id = Uuid::new_v4();

            // 4. Filename Safety (Truncate to 50 chars to prevent DB errors)
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

            // 5. Spawn Blocking Logic (CPU Intensive Image Processing)
            let storage_root = data.env.storage_root.clone();
            let storage_id_clone = storage.id;
            let filename_clone = filename.clone(); // Use original for disk logic to preserve extension
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

            // 6. MIME Type Override & Safety
            let mut db_file_type = if file_type == "raw" { mime_type } else { file_type };
            if db_file_type.len() > 50 {
                db_file_type = "application/octet-stream".to_string();
            }

            // 7. Insert File Record
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
            .bind(&safe_filename)
            .bind(&current_path)
            .bind(final_size)
            .bind(db_file_type)
            .bind(blurhash_opt)
            .fetch_one(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

            // 8. Update Storage Size
            sqlx::query("UPDATE file_storages SET current_size = current_size + $1 WHERE id = $2")
                .bind(final_size)
                .bind(storage.id)
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
    FileStorageApiKeyAuth(storage): FileStorageApiKeyAuth,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    // 1. Fetch Target Item (Scoped to the storage bucket from API Key)
    let target = sqlx::query_as::<_, FileItem>("SELECT * FROM files WHERE id = $1 AND file_storage_id = $2")
        .bind(file_id)
        .bind(storage.id)
        .fetch_optional(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"message": "File not found or access denied"}))))?;

    // Capture values needed for logic/queries before moving target
    let target_storage_id = target.file_storage_id;
    let is_folder = target.is_folder;
    
    // Construct robust full path for folder logic (Matches logic in file_storage.rs)
    let folder_full_path = if is_folder {
        Some(if target.path == "/" {
            target.name.clone()
        } else {
            // Trim trailing slash to prevent double slash issues like "/docs//sub"
            format!("{}/{}", target.path.trim_end_matches('/'), target.name)
        })
    } else {
        None
    };

    // 2. Determine Scope of Deletion (Target + Recursive Descendants)
    let mut files_to_delete = vec![target];
    
    // If it's a folder, we must find everything inside it
    if is_folder {
        if let Some(folder_path) = folder_full_path {
            // Find files that are DIRECT children (path = /folder) 
            // OR recursive children (path LIKE /folder/%)
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

    // 3. Physical Deletion Loop
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
            // Attempt deletion, log errors but don't stop (orphan cleanup)
            if let Err(e) = FileOps::delete_file(&physical_path) {
                tracing::warn!("Failed to delete physical file {}: {}", file.id, e);
            }
            total_size_freed += file.size;
        }
    }

    // 4. Batch Delete Database Records
    sqlx::query("DELETE FROM files WHERE id = ANY($1)")
        .bind(&ids_to_delete)
        .execute(&data.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    // 5. Update Storage Quota
    if total_size_freed > 0 {
        sqlx::query("UPDATE file_storages SET current_size = current_size - $1 WHERE id = $2")
            .bind(total_size_freed)
            .bind(storage.id)
            .execute(&data.db).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("Failed to subtract size from storage: {}", e.to_string())}))))?;
    }

    Ok(Json(json!({"status": "success", "message": format!("Deleted {} item(s)", ids_to_delete.len())})))
}