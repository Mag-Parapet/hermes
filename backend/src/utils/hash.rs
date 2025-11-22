use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: impl Into<String>) -> Result<String, String> {
    let password = password.into();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    Ok(argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}