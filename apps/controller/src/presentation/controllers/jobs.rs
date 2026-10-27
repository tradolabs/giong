use crate::presentation::dtos::requests::get_logs::GetLogsRequestDto;
use crate::presentation::exceptions::http_exception::{HttpException, NotFoundException};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use giong_core::application::application::Application;
use std::sync::Arc;
use uuid::Uuid;

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
