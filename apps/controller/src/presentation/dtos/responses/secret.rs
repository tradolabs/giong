use giong_core::application::services::secret_service::SecretInfo;
use serde::Serialize;
#[derive(Serialize)]
pub(in crate::presentation) struct SecretResponseDto {
    pub key: String,
    pub source: Option<String>,
}
impl From<SecretInfo> for SecretResponseDto {
    fn from(info: SecretInfo) -> Self {
        Self { key: info.key, source: info.source }
    }
}
