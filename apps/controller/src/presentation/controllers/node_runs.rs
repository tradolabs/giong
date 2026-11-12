use crate::presentation::dtos::requests::get_logs::GetLogsRequestDto;
use crate::presentation::exceptions::http_exception::{HttpException, NotFoundException};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::get_node_run_logs::{
    GetNodeRunLogsInput, GetNodeRunLogsUseCase,
};
use std::sync::Arc;
use uuid::Uuid;
pub(in crate::presentation) async fn logs(
    State(application): State<Arc<Application>>,
    Path(run_id): Path<Uuid>,
    Query(query): Query<GetLogsRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    application
        .execute::<GetNodeRunLogsUseCase>(GetNodeRunLogsInput {
            run_id,
            from_lines: query.from_lines,
        })
        .await?
        .ok_or(NotFoundException("Node run not found"))
}
