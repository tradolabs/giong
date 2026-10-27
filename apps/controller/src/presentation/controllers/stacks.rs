use crate::presentation::dtos::responses::{job::JobResponseDto, stack::StackResponseDto};
use crate::presentation::exceptions::http_exception::HttpException;
use axum::{
    Json,
    extract::{Path, State},
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
