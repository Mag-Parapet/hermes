use serde::{Deserialize, Serialize};
use crate::dtos::pagination::PaginationMeta;
use crate::models::file_storage::FileStorage;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileStorageSchema {
    pub name: String,
    pub file_max_size: Option<i64>,
    pub compression_enabled: Option<bool>,
    pub img_resize_max_size: Option<i32>,
    pub img_default_format: Option<String>,
    pub allowed_file_types: Option<String>,
    pub is_active: Option<bool>,
    pub quota_size: Option<i64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFileStorageSchema {
    pub name: Option<String>,
    pub file_max_size: Option<i64>,
    pub compression_enabled: Option<bool>,
    pub img_resize_max_size: Option<i32>,
    pub img_default_format: Option<String>,
    pub allowed_file_types: Option<String>,
    pub is_active: Option<bool>,
    pub quota_size: Option<i64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileStorageFilterOptions {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedFileStorageList {
    pub file_storages: Vec<FileStorage>,
    pub pagination: PaginationMeta,
}