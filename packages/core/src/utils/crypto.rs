use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub(crate) async fn random_password() -> Result<(String, String)> {
    tokio::task::spawn_blocking(|| -> Result<(String, String)> {
        let mut password_bytes = [0u8; 24];
        getrandom::fill(&mut password_bytes)?;
        let password = URL_SAFE_NO_PAD.encode(password_bytes);
        let password_hash = Argon2::default().hash_password(password.as_bytes())?.to_string();
        Ok((password, password_hash))
    })
    .await?
}

pub(crate) async fn verify_password(password: String, password_hash: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || -> Result<bool> {
        let hash = PasswordHash::new(&password_hash)?;
        match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::PasswordInvalid) => Ok(false),
            Err(error) => Err(error.into()),
        }
    })
    .await?
}
