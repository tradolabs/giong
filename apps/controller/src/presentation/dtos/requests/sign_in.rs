use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::presentation) struct SignInRequestDto {
    pub username: String,
    pub password: String,
}
