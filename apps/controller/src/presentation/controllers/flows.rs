use crate::presentation::dtos::responses::flow::FlowResponseDto;
use crate::presentation::exceptions::http_exception::HttpException;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::upsert_flow::{UpsertFlowInput, UpsertFlowUseCase};
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
    Json(input): Json<crate::presentation::dtos::requests::create_flow::CreateFlowRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let flow = application
        .execute::<UpsertFlowUseCase>(UpsertFlowInput {
            id: None,
            name: input.name.trim().to_owned(),
            nodes: input.nodes,
            inherited_vault_ids: input.inherited_vault_ids,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(FlowResponseDto::from(flow))))
}

pub(in crate::presentation) async fn update_flow(
    State(application): State<Arc<Application>>,
    Path(flow_id): Path<Uuid>,
    Json(input): Json<crate::presentation::dtos::requests::update_flow::UpdateFlowRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let flow = application
        .execute::<UpsertFlowUseCase>(UpsertFlowInput {
            id: Some(flow_id),
            name: input.name,
            nodes: input.nodes,
            inherited_vault_ids: input.inherited_vault_ids,
        })
        .await?;
    Ok(Json(FlowResponseDto::from(flow)))
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct RunFlowRequest {
    flow: giong_node::types::node::Flow,
}
pub(in crate::presentation) async fn run(
    State(app): State<Arc<Application>>,
    Json(input): Json<RunFlowRequest>,
) -> Result<impl IntoResponse, HttpException> {
    use giong_core::application::use_cases::run_flow::{RunFlowInput, RunFlowUseCase};
    Ok(Json(
        app.execute::<RunFlowUseCase>(RunFlowInput {
            flow: input.flow,
            run_id: None,
            log_timestamp: None,
            node_id: None,
            scope: None,
        })
        .await?,
    ))
}

pub(in crate::presentation) async fn run_node(
    State(app): State<Arc<Application>>,
    Path(input): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HttpException> {
    use giong_core::application::use_cases::node_triggers::TriggerNodeUseCase;
    let run = app.execute::<TriggerNodeUseCase>(input).await?;
    Ok((StatusCode::ACCEPTED, Json(run)))
}

pub(in crate::presentation) async fn delete_flow(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    use giong_core::application::use_cases::delete_flow::DeleteFlowUseCase;
    app.execute::<DeleteFlowUseCase>(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct UpdateFlowVaultsRequest {
    inherited_vault_ids: Vec<Uuid>,
}
pub(in crate::presentation) async fn update_vaults(
    State(app): State<Arc<Application>>,
    Path(flow_id): Path<Uuid>,
    Json(input): Json<UpdateFlowVaultsRequest>,
) -> Result<impl IntoResponse, HttpException> {
    use giong_core::application::use_cases::update_flow_vaults::{
        UpdateFlowVaultsInput, UpdateFlowVaultsUseCase,
    };
    let flow = app
        .execute::<UpdateFlowVaultsUseCase>(UpdateFlowVaultsInput {
            flow_id,
            inherited_vault_ids: input.inherited_vault_ids,
        })
        .await?;
    Ok(Json(FlowResponseDto::from(flow)))
}

pub(in crate::presentation) async fn runs(
    State(app): State<Arc<Application>>,
    Path(flow_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    use giong_core::application::use_cases::get_node_runs::GetNodeRunsUseCase;
    Ok(Json(app.execute::<GetNodeRunsUseCase>(flow_id).await?))
}
