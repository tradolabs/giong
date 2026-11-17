use crate::presentation::dtos::{
    requests::sign_in::SignInRequestDto, responses::sign_in::SignInResponseDto,
};
use crate::presentation::exceptions::http::HttpException;
use crate::presentation::extractors::validated_json::ValidatedJson;
use axum::{Json, extract::State, response::IntoResponse};
use giong_core::application::application::Application;
use std::sync::Arc;

pub(in crate::presentation) async fn sign_in(
    State(application): State<Arc<Application>>,
    ValidatedJson(input): ValidatedJson<SignInRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let output = application.sign_in(&input.username, &input.password).await?;
    Ok(Json(SignInResponseDto::from(output)))
}
