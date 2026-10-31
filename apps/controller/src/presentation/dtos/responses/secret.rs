use serde::Serialize;

#[derive(Serialize)]
pub(in crate::presentation) struct SecretResponseDto {
    pub key: String,
}

impl From<String> for SecretResponseDto {
    fn from(key: String) -> Self {
        Self { key }
    }
}
