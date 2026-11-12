use super::authentication::AuthenticationException;

#[derive(Debug, thiserror::Error)]
pub enum Exception {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(&'static str),
    #[error("{0}")]
    Conflict(&'static str),
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

impl From<serde_json::Error> for Exception {
    fn from(error: serde_json::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}
