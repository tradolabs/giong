use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
};
use anyhow::{Result, anyhow, ensure};

pub(crate) fn encrypt(key: &[u8], data: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| anyhow!("Encryption key must contain exactly 32 bytes"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, Payload { msg: data, aad: associated_data })
        .map_err(|_| anyhow!("Encryption failed"))?;
    let mut data = nonce.to_vec();
    data.extend_from_slice(&ciphertext);
    Ok(data)
}

pub(crate) fn decrypt(key: &[u8], data: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
    const NONCE_LENGTH: usize = 12;
    ensure!(data.len() >= NONCE_LENGTH + 16, "Invalid encrypted data");
    Aes256Gcm::new_from_slice(key)
        .map_err(|_| anyhow!("Encryption key must contain exactly 32 bytes"))?
        .decrypt(
            Nonce::from_slice(&data[..NONCE_LENGTH]),
            Payload { msg: &data[NONCE_LENGTH..], aad: associated_data },
        )
        .map_err(|_| anyhow!("Decryption failed"))
}
