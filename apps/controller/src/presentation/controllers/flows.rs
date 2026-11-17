use crate::presentation::dtos::requests::create_flow::CreateFlowRequestDto;
use crate::presentation::dtos::requests::update_flow::UpdateFlowRequestDto;
use crate::presentation::dtos::responses::flow::FlowResponseDto;
use crate::presentation::exceptions::http::HttpException;
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
use giong_database::entities::flow_entity::FlowEntity;
use std::sync::Arc;
use uuid::Uuid;

fn flow_response(
    application: &Application,
    flow: FlowEntity,
) -> Result<FlowResponseDto, HttpException> {
    let mut response = FlowResponseDto::from(flow);
    for node in &mut response.nodes {
        node.set_execution(application.get_latest_node_execution(node.id)?);
    }
    Ok(response)
}

pub(in crate::presentation) async fn get_flows(
    State(application): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    let flows = application.get_flows().await?;
    let responses = flows
        .into_iter()
        .map(|flow| flow_response(&application, flow))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(responses))
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
    Ok((StatusCode::CREATED, Json(flow_response(&application, flow)?)))
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
    Ok(Json(flow_response(&application, flow)?))
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
