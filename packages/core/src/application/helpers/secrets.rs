use crate::application::exceptions::base::Exception;
use crate::config::Config;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use giong_database::Database;
use giong_database::entities::vault_entity::VaultEntity;
use giong_database::repositories::types::condition::Condition;
use std::collections::HashSet;
use uuid::Uuid;

pub async fn get_vault(database: &Database, id: Uuid) -> Result<VaultEntity, Exception> {
    database
        .vaults
        .find_one(Condition::new().eq("id", id))
        .await?
        .ok_or(Exception::NotFound("Vault not found"))
}
pub async fn validate_vaults(
    database: &Database,
    local: Option<Uuid>,
    inherited: &[Uuid],
) -> Result<(), Exception> {
    let mut seen = HashSet::new();
    for id in inherited.iter().copied().chain(local) {
        if !seen.insert(id) {
            return Err(Exception::BadRequest("Duplicate vault".into()));
        }
        get_vault(database, id).await?;
    }
    Ok(())
}
fn cipher(config: &Config) -> Result<Aes256Gcm> {
    let key = STANDARD
        .decode(config.encryption_key.as_deref().context("Encryption key is required")?.trim())?;
    Aes256Gcm::new_from_slice(&key)
        .map_err(|_| anyhow::anyhow!("Encryption key must contain 32 bytes"))
}
pub fn encrypt(config: &Config, vault: Uuid, key: &str, value: &str) -> Result<String> {
    let mut nonce = [0; 12];
    getrandom::fill(&mut nonce)?;
    let context = format!("vault:{vault}:{key}");
    let encrypted = cipher(config)?
        .encrypt(&Nonce::from(nonce), Payload { msg: value.as_bytes(), aad: context.as_bytes() })
        .map_err(|_| anyhow::anyhow!("Secret encryption failed"))?;
    let mut bytes = nonce.to_vec();
    bytes.extend_from_slice(&encrypted);
    Ok(STANDARD.encode(bytes))
}
pub fn decrypt(config: &Config, vault: Uuid, key: &str, value: &str) -> Result<String> {
    let bytes = STANDARD.decode(value)?;
    ensure!(bytes.len() >= 28, "Invalid encrypted secret");
    let nonce: [u8; 12] = bytes[..12].try_into()?;
    let context = format!("vault:{vault}:{key}");
    let plaintext = cipher(config)?
        .decrypt(&Nonce::from(nonce), Payload { msg: &bytes[12..], aad: context.as_bytes() })
        .map_err(|_| anyhow::anyhow!("Secret decryption failed"))?;
    Ok(String::from_utf8(plaintext)?)
}
