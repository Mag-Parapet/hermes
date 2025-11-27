use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{models::file::FileItem, dtos::pagination::PaginationMeta}; 

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileSchema {
    pub file_storage_id: Uuid,
    pub name: String,
    pub path: String,
    pub is_folder: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderSchema {
    pub name: String,
    pub path: String, 
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileFilterOptions {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageVariants {
    pub original: String,
    pub blurhash: Option<String>,
    pub sm: String,
    pub md: String,
    pub lg: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResponse {
    pub id: Uuid,
    pub file_storage_id: Uuid,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub file_type: String,
    pub is_folder: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    
    pub url: String,
    pub variants: Option<ImageVariants>, 
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedFileList {
    pub files: Vec<FileResponse>, 
    pub pagination: PaginationMeta,
    pub current_path: String,
}