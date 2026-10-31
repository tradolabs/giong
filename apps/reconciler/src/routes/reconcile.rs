use crate::server::AppState;
use crate::snapshot::Snapshot;
use anyhow::Context;
use axum::{Json, body::Body, extract::State, http::StatusCode, response::IntoResponse};
use giong_provider::protocol::{ChangeStatus, Provider, Resource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type RouteError = (StatusCode, String);

#[derive(Deserialize)]
pub(crate) struct ReconcileRequest {
    #[serde(rename = "stackId", alias = "stack_id")]
    stack_id: String,
    #[serde(default)]
    providers: Vec<Provider>,
    resources: Vec<Resource>,
}

#[derive(Default, Serialize)]
pub(crate) struct ReconcileResponse {
    created: Vec<Resource>,
    updated: Vec<Resource>,
    unchanged: Vec<Resource>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum ReconcileEvent {
    Log(String),
    Error(String),
    Complete(ReconcileResponse),
}

impl ReconcileEvent {
    fn encode(self) -> Result<String, serde_json::Error> {
        match &self {
            Self::Log(message) | Self::Error(message) => eprintln!("{message}"),
            Self::Complete(_) => {}
        }
        serde_json::to_string(&self).map(|json| json + "\n")
    }
}

pub(crate) async fn reconcile(
    State(state): State<AppState>,
    Json(input): Json<ReconcileRequest>,
) -> Result<impl IntoResponse, RouteError> {
    let providers: HashMap<_, _> =
        input.providers.into_iter().map(|provider| (provider.urn.clone(), provider)).collect();
    let storage_blob = format!("snapshots/{}", input.stack_id);
    let mut snapshot = Snapshot::load(&state.storage, &storage_blob)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, format!("Could not load snapshot: {error}")))?;

    let stream = async_stream::stream! {
        let mut response = ReconcileResponse::default();
        for resource in input.resources {
            let urn = resource.urn();
            let secrets = resource.provider.as_ref()
                .and_then(|id| providers.get(id)).map(|it| &it.inputs);
            yield ReconcileEvent::Log(format!("Reconciling {urn}")).encode();

            let change = match reconcile_and_save(&state, &mut snapshot, &storage_blob, &resource, secrets).await {
                Ok(change) => change,
                Err(error) => {
                    yield ReconcileEvent::Error(format!("{urn}: {error:#}")).encode();
                    return;
                }
            };
            yield ReconcileEvent::Log(format!("{urn} {change:?}")).encode();

            match change {
                ChangeStatus::Created => response.created.push(resource),
                ChangeStatus::Updated => response.updated.push(resource),
                ChangeStatus::Unchanged => response.unchanged.push(resource),
            }
        }
        yield ReconcileEvent::Complete(response).encode();
    };

    Ok((
        [("content-type", "application/x-ndjson"), ("x-accel-buffering", "no")],
        Body::from_stream(stream),
    ))
}

async fn reconcile_and_save(
    state: &AppState,
    snapshot: &mut Snapshot,
    key: &str,
    resource: &Resource,
    secrets: Option<&serde_json::Value>,
) -> anyhow::Result<ChangeStatus> {
    let change = state
        .runtime
        .read(&resource.package, &resource.inputs, secrets)
        .await
        .context("read failed")?;
    if change != ChangeStatus::Unchanged {
        state
            .runtime
            .apply(&resource.package, &resource.inputs, secrets)
            .await
            .context("apply failed")?;
    }
    snapshot.insert(resource);
    snapshot.save(&state.storage, key).await.context("Could not save snapshot")?;
    Ok(change)
}
