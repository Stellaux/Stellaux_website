//! Password hashing via Argon2id (OWASP-recommended). Salt is per-password,
//! generated from the OS RNG; the encoded hash carries its own salt + params.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::common::error::{AppError, AppResult};

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 hash failed: {e}")))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, encoded_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(encoded_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid hash format: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
