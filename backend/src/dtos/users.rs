use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::user::UserRole;

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
    pub name: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}