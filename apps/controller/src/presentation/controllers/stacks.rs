use crate::presentation::dtos::requests::create_secret::CreateSecretRequestDto;
use crate::presentation::dtos::requests::update_secret::UpdateSecretRequestDto;
use crate::presentation::dtos::responses::job::JobResponseDto;
use crate::presentation::dtos::responses::secret::SecretResponseDto;
use crate::presentation::dtos::responses::stack::StackResponseDto;
use crate::presentation::exceptions::http_exception::HttpException;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use std::sync::Arc;
use uuid::Uuid;

pub(in crate::presentation) async fn get_stacks(
    State(application): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    let stacks = application.get_stacks().await?;
    Ok(Json(stacks.into_iter().map(StackResponseDto::from).collect::<Vec<_>>()))
}

pub(in crate::presentation) async fn get_jobs(
    State(application): State<Arc<Application>>,
    Path(stack_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    let jobs = application.get_jobs(stack_id).await?;
    Ok(Json(jobs.into_iter().map(JobResponseDto::from).collect::<Vec<_>>()))
}

pub(in crate::presentation) async fn get_secrets(
    State(application): State<Arc<Application>>,
    Path(stack_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    let secrets = application.get_secrets(stack_id).await?;
    Ok(Json(secrets.into_iter().map(SecretResponseDto::from).collect::<Vec<_>>()))
}

pub(in crate::presentation) async fn create_secret(
    State(application): State<Arc<Application>>,
    Path(stack_id): Path<Uuid>,
    Json(input): Json<CreateSecretRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    application.create_secret(stack_id, input.key, input.value).await?;
    let secrets = application.get_secrets(stack_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(secrets.into_iter().map(SecretResponseDto::from).collect::<Vec<_>>()),
    ))
}

pub(in crate::presentation) async fn update_secret(
    State(application): State<Arc<Application>>,
    Path((stack_id, key)): Path<(Uuid, String)>,
    Json(input): Json<UpdateSecretRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    application.update_secret(stack_id, key, input.value).await?;
    let secrets = application.get_secrets(stack_id).await?;
    Ok(Json(secrets.into_iter().map(SecretResponseDto::from).collect::<Vec<_>>()))
}

pub(in crate::presentation) async fn delete_secret(
    State(application): State<Arc<Application>>,
    Path((stack_id, key)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, HttpException> {
    application.delete_secret(stack_id, key).await?;
    let secrets = application.get_secrets(stack_id).await?;
    Ok(Json(secrets.into_iter().map(SecretResponseDto::from).collect::<Vec<_>>()))
}
