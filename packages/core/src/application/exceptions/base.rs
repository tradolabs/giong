use super::authentication::AuthenticationException;

#[derive(Debug, thiserror::Error)]
pub enum Exception {
    #[error(transparent)]
    Authentication(#[from] AuthenticationException),
    #[error("{0}")]
    Unexpected(String),
}

impl Exception {
    pub fn unexpected(message: impl ToString) -> Self {
        Self::Unexpected(message.to_string())
    }
}

impl From<anyhow::Error> for Exception {
    fn from(error: anyhow::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_database::error::Error> for Exception {
    fn from(error: giong_database::error::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_storage::Error> for Exception {
    fn from(error: giong_storage::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_git::Error> for Exception {
    fn from(error: giong_git::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}
