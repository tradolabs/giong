use crate::presentation::dtos::requests::create_flow::CreateFlowRequestDto;
use crate::presentation::dtos::requests::update_flow::UpdateFlowRequestDto;
use crate::presentation::dtos::responses::flow::FlowResponseDto;
use crate::presentation::exceptions::http_exception::HttpException;
use crate::presentation::extractors::validated_json::ValidatedJson;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::create_flow::CreateFlowInput;
use giong_core::application::use_cases::update_flow::UpdateFlowInput;
use std::sync::Arc;
use uuid::Uuid;

pub(in crate::presentation) async fn get_flows(
    State(application): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    let flows = application.get_flows().await?;
    Ok(Json(flows.into_iter().map(FlowResponseDto::from).collect::<Vec<_>>()))
}

pub(in crate::presentation) async fn create_flow(
    State(application): State<Arc<Application>>,
    ValidatedJson(input): ValidatedJson<CreateFlowRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let flow = application
        .create_flow(CreateFlowInput {
            name: input.name.trim().to_owned(),
            inherited_vault_ids: input.inherited_vault_ids,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(FlowResponseDto::from(flow))))
}

pub(in crate::presentation) async fn update_flow(
    State(application): State<Arc<Application>>,
    Path(flow_id): Path<Uuid>,
    ValidatedJson(input): ValidatedJson<UpdateFlowRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let flow = application
        .update_flow(UpdateFlowInput {
            id: flow_id,
            name: input.name,
            inherited_vault_ids: input.inherited_vault_ids,
        })
        .await?;
    Ok(Json(FlowResponseDto::from(flow)))
}

pub(in crate::presentation) async fn execute_node(
    State(app): State<Arc<Application>>,
    Path((flow_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HttpException> {
    let execution = app.execute_node(flow_id, node_id).await?;
    Ok((StatusCode::ACCEPTED, Json(execution)))
}

pub(in crate::presentation) async fn delete_flow(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    app.delete_flow(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
