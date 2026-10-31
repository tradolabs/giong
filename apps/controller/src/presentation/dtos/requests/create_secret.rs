use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::presentation) struct CreateSecretRequestDto {
    pub key: String,
    pub value: String,
}
