use crate::presentation::dtos::requests::create_node::CreateNodeRequestDto;
use crate::presentation::dtos::requests::get_logs::GetLogsRequestDto;
use crate::presentation::dtos::requests::update_node::UpdateNodeRequestDto;
use crate::presentation::dtos::responses::node::NodeResponseDto;
use crate::presentation::exceptions::http_exception::{HttpException, NotFoundException};
use crate::presentation::extractors::validated_json::ValidatedJson;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::create_node::CreateNodeInput;
use giong_core::application::use_cases::get_node_logs::GetNodeLogsInput;
use giong_core::application::use_cases::update_node::UpdateNodeInput;
use std::sync::Arc;
use uuid::Uuid;

pub(in crate::presentation) async fn get_details(
    State(application): State<Arc<Application>>,
    Path((flow_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(application.get_node_details(flow_id, node_id).await?))
}

pub(in crate::presentation) async fn delete_node(
    State(application): State<Arc<Application>>,
    Path((flow_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HttpException> {
    application.delete_node(flow_id, node_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub(in crate::presentation) async fn get_logs(
    State(application): State<Arc<Application>>,
    Path(log_id): Path<Uuid>,
    Query(query): Query<GetLogsRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    application
        .get_node_logs(GetNodeLogsInput { execution_id: log_id, from_lines: query.from_lines })
        .await?
        .ok_or(NotFoundException("Log not found"))
}

pub(in crate::presentation) async fn create_node(
    State(application): State<Arc<Application>>,
    Path(flow_id): Path<Uuid>,
    ValidatedJson(input): ValidatedJson<CreateNodeRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let node = application
        .create_node(CreateNodeInput {
            flow_id,
            name: input.name,
            kind: input.kind,
            properties: input.properties,
            triggers: input.triggers,
        })
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(NodeResponseDto::from(node))))
}

pub(in crate::presentation) async fn update_node(
    State(application): State<Arc<Application>>,
    Path((flow_id, node_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(input): ValidatedJson<UpdateNodeRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let node = application
        .update_node(UpdateNodeInput {
            flow_id,
            node_id,
            name: input.name,
            kind: input.kind,
            properties: input.properties,
            triggers: input.triggers,
        })
        .await?;
    Ok(Json(NodeResponseDto::from(node)))
}
