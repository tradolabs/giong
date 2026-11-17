use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use uuid::Uuid;

fn secret_cipher(encryption_key: Option<&str>) -> Result<Aes256Gcm> {
    let key = STANDARD.decode(encryption_key.context("Encryption key is required")?.trim())?;
    Aes256Gcm::new_from_slice(&key)
        .map_err(|_| anyhow::anyhow!("Encryption key must contain 32 bytes"))
}

pub(crate) fn encrypt_secret(
    encryption_key: Option<&str>,
    vault_id: Uuid,
    key: &str,
    value: &str,
) -> Result<String> {
    let cipher = secret_cipher(encryption_key)?;
    let mut nonce = [0; 12];
    getrandom::fill(&mut nonce)?;
    let context = format!("vault:{vault_id}:{key}");
    let encrypted = cipher
        .encrypt(&Nonce::from(nonce), Payload { msg: value.as_bytes(), aad: context.as_bytes() })
        .map_err(|_| anyhow::anyhow!("Secret encryption failed"))?;
    let mut bytes = Vec::with_capacity(12 + encrypted.len());
    bytes.extend_from_slice(&nonce);
    bytes.extend_from_slice(&encrypted);
    Ok(STANDARD.encode(bytes))
}

pub(crate) fn decrypt_secret(
    encryption_key: Option<&str>,
    vault_id: Uuid,
    key: &str,
    encrypted: &str,
) -> Result<String> {
    let bytes = STANDARD.decode(encrypted)?;
    if bytes.len() < 28 {
        anyhow::bail!("Invalid encrypted secret");
    }
    let nonce: [u8; 12] = bytes[..12].try_into()?;
    let cipher = secret_cipher(encryption_key)?;
    let context = format!("vault:{vault_id}:{key}");
    let plaintext = cipher
        .decrypt(&Nonce::from(nonce), Payload { msg: &bytes[12..], aad: context.as_bytes() })
        .map_err(|_| anyhow::anyhow!("Secret decryption failed"))?;
    Ok(String::from_utf8(plaintext)?)
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
