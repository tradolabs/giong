use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct CreateSecretRequestDto {
    #[validate(length(min = 1))]
    pub key: String,
    pub value: String,
}
