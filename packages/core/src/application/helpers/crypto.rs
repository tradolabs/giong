use crate::utils::crypto::{decrypt, encrypt};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};

pub(crate) fn encrypt_token(value: &str, key: &str, associated_data: &[u8]) -> Result<String> {
    let key = STANDARD
        .decode(key.trim())
        .context("Encryption key must be a base64-encoded 32-byte key")?;
    ensure!(key.len() == 32, "Encryption key must be a base64-encoded 32-byte key");
    let encrypted = encrypt(&key, value.as_bytes(), associated_data)?;
    Ok(format!("enc:v1:{}", STANDARD.encode(encrypted)))
}

pub(crate) fn decrypt_token(value: &str, key: &str, associated_data: &[u8]) -> Result<String> {
    let encoded =
        value.strip_prefix("enc:v1:").context("Unsupported credential encryption format")?;
    let data = STANDARD.decode(encoded)?;
    let key = STANDARD
        .decode(key.trim())
        .context("Encryption key must be a base64-encoded 32-byte key")?;
    ensure!(key.len() == 32, "Encryption key must be a base64-encoded 32-byte key");
    let decrypted = decrypt(&key, &data, associated_data)?;
    Ok(String::from_utf8(decrypted)?)
}
