use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub id: Uuid,
    pub user_id: Uuid,
    pub domain: String,
    pub port: i32,
    pub max_body_size: Option<i32>,
    pub is_ssl: Option<bool>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}