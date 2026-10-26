use crate::presentation::exceptions::http_exception::HttpException;
use axum::{Json, extract::State, response::IntoResponse};
use giong_core::application::application::Application;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub(in crate::presentation) struct SignInRequest {
    username: String,
    password: String,
}

pub(in crate::presentation) async fn sign_in(
    State(application): State<Arc<Application>>,
    Json(input): Json<SignInRequest>,
) -> Result<impl IntoResponse, HttpException> {
    let output = application.sign_in(&input.username, &input.password).await?;

    Ok(Json(output))
}
