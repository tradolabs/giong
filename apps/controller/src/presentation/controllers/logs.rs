use crate::presentation::exceptions::http_exception::HttpException;
use crate::presentation::exceptions::http_exception::NotFoundException;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use giong_core::application::application::Application;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub(in crate::presentation) struct LogsQuery {
    #[serde(default)]
    from_lines: usize,
}

pub(in crate::presentation) async fn get_logs(
    State(application): State<Arc<Application>>,
    Path(job_id): Path<Uuid>,
    Query(query): Query<LogsQuery>,
) -> Result<impl IntoResponse, HttpException> {
    application
        .get_job_logs(&job_id, query.from_lines)
        .await?
        .ok_or(NotFoundException("modules.job.not_found"))
}
