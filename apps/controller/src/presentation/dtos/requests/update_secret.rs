use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::presentation) struct UpdateSecretRequestDto {
    pub value: String,
}
