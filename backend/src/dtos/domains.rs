use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{models::domain, dtos::pagination::PaginationMeta};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateDomainSchema {
    pub domain: String,
    pub port: i32,
    pub max_body_size: Option<i32>,
    pub is_ssl: Option<bool>,
    pub is_active: Option<bool>,
    /// Path to SSL certificate (e.g., /etc/letsencrypt/live/domain/fullchain.pem)
    /// If not provided and is_ssl=true, defaults to Let's Encrypt path
    pub ssl_certificate_path: Option<String>,
    /// Path to SSL private key (e.g., /etc/letsencrypt/live/domain/privkey.pem)
    /// If not provided and is_ssl=true, defaults to Let's Encrypt path
    pub ssl_certificate_key_path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDomainSchema {
    pub domain: String,
    pub port: i32,
    pub max_body_size: Option<i32>,
    pub is_ssl: Option<bool>,
    pub is_active: Option<bool>,
    pub ssl_certificate_path: Option<String>,
    pub ssl_certificate_key_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainResponse {
    pub id: Uuid,
    pub domain: String,
    pub port: i32,
    pub is_active: bool,
    pub is_ssl: bool,
    pub ssl_certificate_path: Option<String>,
    pub ssl_certificate_key_path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DomainFilterOptions {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub port: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedDomainList {
    pub domains: Vec<domain::Domain>,
    pub pagination: PaginationMeta,
}