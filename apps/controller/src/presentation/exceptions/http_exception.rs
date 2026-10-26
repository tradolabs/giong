use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use giong_core::application::exceptions::authentication::AuthenticationException;
use giong_core::application::exceptions::base::Exception;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub(in crate::presentation) enum HttpException {
    #[error(transparent)]
    Application(#[from] Exception),
    #[error("{code}")]
    Response { status: StatusCode, code: &'static str },
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
}

impl IntoResponse for HttpException {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            code: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            errors: Option<HashMap<String, Vec<String>>>,
        }

        let (status, code, errors) = match self {
            Self::Application(exception) => {
                eprintln!("Application exception: {exception:?}");
                match exception {
                    Exception::Authentication(AuthenticationException::InvalidCredentials) => {
                        (StatusCode::UNAUTHORIZED, "modules.auth.invalid_credentials", None)
                    }
                    Exception::Unexpected(_) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, "http.internal_error", None)
                    }
                }
            }
            Self::Response { status, code } => (status, code, None),
            Self::InternalServerError(message) => {
                eprintln!("Internal server error: {message}");
                (StatusCode::INTERNAL_SERVER_ERROR, "http.internal_error", None)
            }
        };
        (status, Json(ErrorBody { code, errors })).into_response()
    }
}

#[allow(non_snake_case)]
pub(in crate::presentation) fn UnauthorizedException(code: &'static str) -> HttpException {
    HttpException::Response { status: StatusCode::UNAUTHORIZED, code }
}

#[allow(non_snake_case)]
pub(in crate::presentation) fn BadRequestException(code: &'static str) -> HttpException {
    HttpException::Response { status: StatusCode::BAD_REQUEST, code }
}

#[allow(non_snake_case)]
pub(in crate::presentation) fn NotFoundException(code: &'static str) -> HttpException {
    HttpException::Response { status: StatusCode::NOT_FOUND, code }
}

#[allow(non_snake_case)]
pub(in crate::presentation) fn ServiceUnavailableException(code: &'static str) -> HttpException {
    HttpException::Response { status: StatusCode::SERVICE_UNAVAILABLE, code }
}
