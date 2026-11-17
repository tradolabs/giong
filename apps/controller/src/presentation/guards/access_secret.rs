use crate::presentation::exceptions::http::HttpException;
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use giong_core::application::application::Application;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

pub(in crate::presentation) struct AccessSecretGuard;

#[derive(Deserialize)]
struct SecretPath {
    vault_id: Uuid,
    secret_id: Uuid,
}

impl FromRequestParts<Arc<Application>> for AccessSecretGuard {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        application: &Arc<Application>,
    ) -> Result<Self, Self::Rejection> {
        let Path(path) = Path::<SecretPath>::from_request_parts(parts, application)
            .await
            .map_err(IntoResponse::into_response)?;
        application
            .get_secret(path.vault_id, path.secret_id)
            .await
            .map_err(|error| HttpException::from(error).into_response())?;
        Ok(Self)
    }
}
