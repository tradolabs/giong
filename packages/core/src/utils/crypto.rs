use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, ensure};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

pub(crate) fn encrypt(key: &[u8], value: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
    let cipher =
        Aes256Gcm::new_from_slice(key).context("Encryption key must contain exactly 32 bytes")?;

    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce)?;

    let mut output = nonce.to_vec();
    output.extend_from_slice(
        &cipher
            .encrypt(&Nonce::from(nonce), Payload { msg: value, aad: associated_data })
            .context("Encryption failed")?,
    );
    Ok(output)
}

pub(crate) fn decrypt(key: &[u8], value: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
    const NONCE_LENGTH: usize = 12;
    ensure!(value.len() >= NONCE_LENGTH + 16, "Invalid encrypted data");
    let nonce: [u8; NONCE_LENGTH] = value[..NONCE_LENGTH].try_into()?;
    Aes256Gcm::new_from_slice(key)
        .context("Encryption key must contain exactly 32 bytes")?
        .decrypt(&Nonce::from(nonce), Payload { msg: &value[NONCE_LENGTH..], aad: associated_data })
        .context("Decryption failed")
}

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
