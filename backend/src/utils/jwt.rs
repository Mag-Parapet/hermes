use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::{models::user::UserRole, dtos::users::TokenClaims};

pub fn generate_tokens(user_id: Uuid, role: UserRole, secret: &str) -> Result<(String, String), String> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    
    let access_exp = (now + Duration::minutes(15)).timestamp() as usize;
    let refresh_exp = (now + Duration::days(7)).timestamp() as usize;
    
    let role_str = format!("{:?}", role).to_lowercase();

    let access_claims = TokenClaims { sub: user_id.to_string(), role: role_str.clone(), exp: access_exp, iat };
    let refresh_claims = TokenClaims { sub: user_id.to_string(), role: role_str, exp: refresh_exp, iat };

    let access_token = encode(&Header::default(), &access_claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|e| e.to_string())?;
    let refresh_token = encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|e| e.to_string())?;

    Ok((access_token, refresh_token))
}

pub fn verify_token(token: &str, secret: &str) -> Result<TokenClaims, String> {
    let token_data = decode::<TokenClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default()).map_err(|e| e.to_string())?;
    Ok(token_data.claims)
}