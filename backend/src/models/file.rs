use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileItem {
    pub id: Uuid,
    pub file_storage_id: Uuid,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub file_type: String,
    pub is_folder: bool,
    pub blurhash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}