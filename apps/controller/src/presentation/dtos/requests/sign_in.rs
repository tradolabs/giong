use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub(in crate::presentation) struct SignInRequestDto {
    #[validate(length(min = 1), non_control_character)]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}
