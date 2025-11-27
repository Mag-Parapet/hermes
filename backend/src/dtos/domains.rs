use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{models::domain::Domain, dtos::pagination::PaginationMeta};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateDomainSchema {
    pub domain: String,
    pub client_max_body_size: Option<i64>,
    pub is_ssl: Option<bool>,
    
    pub domain_type: String, 

    // NGINX FIELDS
    pub nginx_root_path: Option<String>,
    pub nginx_target_host: Option<String>,
    pub nginx_config_content: Option<String>,
    
    // SSL fields
    pub ssl_certificate_path: Option<String>,
    pub ssl_certificate_key_path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDomainSchema {
    pub domain: Option<String>,
    pub client_max_body_size: Option<i64>,
    pub is_ssl: Option<bool>,
    pub is_active: Option<bool>,
    
    pub domain_type: Option<String>,

    // NGINX FIELDS
    pub nginx_root_path: Option<String>,
    pub nginx_target_host: Option<String>,
    pub nginx_config_content: Option<String>,
    
    // SSL fields
    pub ssl_certificate_path: Option<String>,
    pub ssl_certificate_key_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainResponse {
    pub id: Uuid,
    pub domain: String,
    pub is_active: bool,
    pub is_ssl: bool,
    
    pub domain_type: String,

    // NGINX FIELDS
    pub nginx_root_path: Option<String>,
    pub nginx_target_host: Option<String>,
    pub nginx_config_content: Option<String>,
    
    // SSL fields
    pub ssl_certificate_path: Option<String>,
    pub ssl_certificate_key_path: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DomainFilterOptions {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub search: Option<String>,
    pub domain_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedDomainList {
    pub domains: Vec<Domain>,
    pub pagination: PaginationMeta,
}