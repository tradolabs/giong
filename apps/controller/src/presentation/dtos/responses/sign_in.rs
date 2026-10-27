use giong_core::application::use_cases::sign_in::SignInOutput;
use serde::Serialize;

#[derive(Serialize)]
pub(in crate::presentation) struct SignInResponseDto {
    pub access_token: String,
    pub expires_in: u64,
}

impl From<SignInOutput> for SignInResponseDto {
    fn from(output: SignInOutput) -> Self {
        Self { access_token: output.access_token, expires_in: output.expires_in }
    }
}
