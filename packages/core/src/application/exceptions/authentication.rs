#[derive(Debug, thiserror::Error)]
pub enum AuthenticationException {
    #[error("Invalid credentials")]
    InvalidCredentials,
}
