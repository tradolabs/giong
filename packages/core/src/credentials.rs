use crate::utils::crypto;
use anyhow::{Context, Result, anyhow, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::env;

#[derive(Default)]
pub(crate) struct Credentials {
    associated_data: Vec<u8>,
}

impl Credentials {
    pub(crate) fn new(associated_data: &[u8]) -> Self {
        Self { associated_data: associated_data.to_vec() }
    }

    pub(crate) fn encrypt(&self, value: &str) -> Result<String> {
        let encrypted = crypto::encrypt(
            &Self::load_encryption_key()?,
            value.as_bytes(),
            &self.associated_data,
        )
        .context("Could not encrypt credentials")?;
        Ok(format!("enc:v1:{}", STANDARD.encode(encrypted)))
    }

    pub(crate) fn decrypt(&self, value: &str) -> Result<String> {
        let encoded = value
            .strip_prefix("enc:v1:")
            .context("Unsupported credential encryption format")?;
        let encrypted = STANDARD
            .decode(encoded)
            .context("Invalid encrypted credentials")?;
        let decrypted = crypto::decrypt(
            &Self::load_encryption_key()?,
            &encrypted,
            &self.associated_data,
        )
        .context("Could not decrypt credentials; verify GIONG_ENCRYPTION_KEY")?;
        String::from_utf8(decrypted).context("Decrypted credentials are not UTF-8")
    }

    fn load_encryption_key() -> Result<Vec<u8>> {
        let value = env::var("GIONG_ENCRYPTION_KEY").map_err(|_| {
            anyhow!("Set GIONG_ENCRYPTION_KEY in .env or the environment; see .env.example")
        })?;
        let key = STANDARD
            .decode(value.trim())
            .map_err(|_| anyhow!("GIONG_ENCRYPTION_KEY must be a base64-encoded 32-byte key"))?;
        ensure!(
            key.len() == 32,
            "GIONG_ENCRYPTION_KEY must be a base64-encoded 32-byte key"
        );
        Ok(key)
    }
}
