use crate::presentation::dtos::requests::get_logs::GetLogsRequestDto;
use crate::presentation::dtos::responses::job::JobResponseDto;
use crate::presentation::exceptions::http_exception::{HttpException, NotFoundException};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use std::sync::Arc;
use uuid::Uuid;

pub(in crate::presentation) async fn create_job(
    State(application): State<Arc<Application>>,
    Path(stack_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    let job = application.create_job(stack_id).await?;
    Ok((StatusCode::CREATED, Json(JobResponseDto::from(job))))
}

pub(in crate::presentation) async fn get_job_logs(
    State(application): State<Arc<Application>>,
    Path(job_id): Path<Uuid>,
    Query(query): Query<GetLogsRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    application
        .get_job_logs(&job_id, query.from_lines)
        .await?
        .ok_or(NotFoundException("modules.job.not_found"))
}
