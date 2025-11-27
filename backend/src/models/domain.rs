use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub id: Uuid,
    pub user_id: Uuid,
    pub domain: String,
    
    // Nginx standard setting
    pub client_max_body_size: Option<i64>,
    pub is_ssl: Option<bool>,
    pub is_active: Option<bool>,
    
    // ('reverse_proxy', 'web_server', 'static_host', 'custom')
    pub domain_type: String,

    pub nginx_root_path: Option<String>,
    pub nginx_target_host: Option<String>,
    pub nginx_config_content: Option<String>,

    // SSL fields
    pub ssl_certificate_path: Option<String>,
    pub ssl_certificate_key_path: Option<String>,
    
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}