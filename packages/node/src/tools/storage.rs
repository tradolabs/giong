use crate::runtime::{Host, bindings};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use giong_kv::kv::Namespace;

const ENCRYPTED: &[u8] = b"giong:encrypted:v1:";

#[derive(Clone)]
pub struct Storage {
    namespace: Namespace,
    cipher: Option<Aes256Gcm>,
    identity: String,
}

impl Storage {
    pub fn new(
        namespace: Namespace,
        identity: String,
        encryption_key: Option<&str>,
    ) -> Result<Self> {
        let cipher = encryption_key
            .map(|key| -> Result<_> {
                let key = STANDARD.decode(key.trim()).context("Invalid encryption key encoding")?;
                Ok(Aes256Gcm::new_from_slice(&key)
                    .map_err(|_| anyhow::anyhow!("Encryption key must contain 32 bytes"))?)
            })
            .transpose()?;
        Ok(Self { namespace, cipher, identity })
    }

    fn associated_data(&self, key: &str) -> String {
        format!("{}:{}:{}", self.identity.len(), self.identity, key)
    }

    fn encode(&self, key: &str, value: &[u8]) -> Result<Vec<u8>> {
        let cipher =
            self.cipher.as_ref().context("Encryption key is required to write node storage")?;
        let mut nonce = [0; 12];
        getrandom::fill(&mut nonce)?;
        let encrypted = cipher
            .encrypt(
                &Nonce::from(nonce),
                Payload { msg: value, aad: self.associated_data(key).as_bytes() },
            )
            .map_err(|_| anyhow::anyhow!("Storage encryption failed"))?;
        let mut output = ENCRYPTED.to_vec();
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&encrypted);
        Ok(output)
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let Some(value) = self.namespace.get(key.as_bytes())? else {
            return Ok(None);
        };
        let value = value.strip_prefix(ENCRYPTED).context("Invalid storage value encoding")?;
        ensure!(value.len() >= 28, "Invalid encrypted storage value");
        let cipher = self.cipher.as_ref().context("Encryption key is required to read storage")?;
        let nonce: [u8; 12] = value[..12].try_into()?;
        Ok(Some(
            cipher
                .decrypt(
                    &Nonce::from(nonce),
                    Payload { msg: &value[12..], aad: self.associated_data(key).as_bytes() },
                )
                .map_err(|_| anyhow::anyhow!("Storage decryption failed"))?,
        ))
    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        Ok(self.namespace.put(key.as_bytes(), &self.encode(key, value)?)?)
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        Ok(self.namespace.delete(key.as_bytes())?)
    }

    pub fn keys(&self, prefix: &str) -> Result<Vec<String>> {
        self.namespace
            .list(prefix.as_bytes())?
            .into_iter()
            .map(|(key, _)| String::from_utf8(key).context("Storage key is not UTF-8"))
            .collect()
    }
}

impl bindings::giong::node::storage::Host for Host {
    fn get(&mut self, key: String) -> Result<Option<Vec<u8>>, String> {
        self.storage
            .as_ref()
            .ok_or("Storage access is not granted")?
            .get(&key)
            .map_err(|_| "Storage read failed".into())
    }
    fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), String> {
        self.storage
            .as_ref()
            .ok_or("Storage access is not granted")?
            .put(&key, &value)
            .map_err(|_| "Storage write failed".into())
    }
    fn delete(&mut self, key: String) -> Result<bool, String> {
        self.storage
            .as_ref()
            .ok_or("Storage access is not granted")?
            .delete(&key)
            .map_err(|_| "Storage delete failed".into())
    }
    fn keys(&mut self, prefix: String) -> Result<Vec<String>, String> {
        self.storage
            .as_ref()
            .ok_or("Storage access is not granted")?
            .keys(&prefix)
            .map_err(|_| "Storage listing failed".into())
    }
}
