use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::user::UserRole;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterUserSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginUserSchema {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

// ... keep existing code ...

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateDomainSchema {
    pub domain: String,
    pub port: i32,
    pub max_body_size: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainResponse {
    pub id: Uuid,
    pub domain: String,
    pub port: i32,
    pub is_active: bool,
}