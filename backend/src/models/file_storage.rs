use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileStorage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub file_max_size: i64,
    pub compression_enabled: bool,
    pub img_resize_max_size: i32,
    pub img_default_format: String,
    pub allowed_file_types: String,
    pub is_active: bool,
    pub api_key: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}