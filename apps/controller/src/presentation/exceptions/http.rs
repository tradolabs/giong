use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use giong_core::application::exceptions::authentication::AuthenticationException;
use giong_core::application::exceptions::base::Exception;
use giong_core::application::exceptions::resource::{Resource, ResourceException};
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

impl HttpException {
    pub(in crate::presentation) fn get_translation_code(&self) -> &'static str {
        match self {
            Self::Application(exception) => match exception {
                Exception::BadRequest(_) => "http.bad_request",
                Exception::Resource(exception) => match exception {
                    ResourceException::NotFound(Resource::Flow) => "modules.flow.not_found",
                    ResourceException::NotFound(Resource::Node) => "modules.node.not_found",
                    ResourceException::NotFound(Resource::Vault) => "modules.vault.not_found",
                    ResourceException::NotFound(Resource::Secret) => "modules.secret.not_found",
                    ResourceException::AlreadyExists(Resource::Flow) => {
                        "modules.flow.already_exists"
                    }
                    ResourceException::AlreadyExists(Resource::Node) => {
                        "modules.node.already_exists"
                    }
                    ResourceException::AlreadyExists(Resource::Vault) => {
                        "modules.vault.already_exists"
                    }
                    ResourceException::AlreadyExists(Resource::Secret) => {
                        "modules.secret.already_exists"
                    }
                    ResourceException::InUse(Resource::Flow) => "modules.flow.in_use",
                    ResourceException::InUse(Resource::Node) => "modules.node.in_use",
                    ResourceException::InUse(Resource::Vault) => "modules.vault.in_use",
                    ResourceException::InUse(Resource::Secret) => "modules.secret.in_use",
                },
                Exception::Authentication(AuthenticationException::InvalidCredentials) => {
                    "modules.auth.invalid_credentials"
                }
                Exception::Unexpected(_) => "http.internal_error",
            },
            Self::Response { code, .. } => code,
            Self::InternalServerError(_) => "http.internal_error",
        }
    }
}

impl IntoResponse for HttpException {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            code: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            message: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            errors: Option<HashMap<String, Vec<String>>>,
        }

        let code = self.get_translation_code();
        let mut message = None;
        let status = match self {
            Self::Application(exception) => {
                eprintln!("Application exception: {exception:?}");
                match exception {
                    Exception::BadRequest(reason) => {
                        message = Some(reason);
                        StatusCode::BAD_REQUEST
                    }
                    Exception::Resource(ResourceException::NotFound(_)) => StatusCode::NOT_FOUND,
                    Exception::Resource(
                        ResourceException::AlreadyExists(_) | ResourceException::InUse(_),
                    ) => StatusCode::CONFLICT,
                    Exception::Authentication(_) => StatusCode::UNAUTHORIZED,
                    Exception::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
            Self::Response { status, .. } => status,
            Self::InternalServerError(reason) => {
                eprintln!("Internal server error: {reason}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, Json(ErrorBody { code, message, errors: None })).into_response()
    }
}

#[allow(non_snake_case)]
pub(in crate::presentation) fn UnauthorizedException(code: &'static str) -> HttpException {
    HttpException::Response { status: StatusCode::UNAUTHORIZED, code }
}

#[allow(non_snake_case)]
pub(in crate::presentation) fn NotFoundException(code: &'static str) -> HttpException {
    HttpException::Response { status: StatusCode::NOT_FOUND, code }
}
