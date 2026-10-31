use crate::server::AppState;
use crate::snapshot::{self, Snapshot};
use anyhow::Context;
use axum::{Json, body::Body, extract::State, http::StatusCode, response::IntoResponse};
use giong_provider::protocol::{ChangeStatus, Resource};
use serde::{Deserialize, Serialize};

type RouteError = (StatusCode, String);

#[derive(Deserialize)]
pub(crate) struct ReconcileRequest {
    stack_id: String,
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
    let storage_blob = snapshot::storage_key(&input.stack_id)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid stack_id".into()))?;

    let mut snapshot = Snapshot::load(&state.storage, &storage_blob)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, format!("Could not load snapshot: {error}")))?;

    let stream = async_stream::stream! {
        let mut response = ReconcileResponse::default();
        for mut resource in input.resources {
            let label = format!("Resource {:?} ({})", resource.name, resource.resource_type);
            yield ReconcileEvent::Log(format!("Reconciling {label}")).encode();

            let change = match reconcile_and_save(&state, &mut snapshot, &storage_blob, &mut resource).await {
                Ok(change) => change,
                Err(error) => {
                    yield ReconcileEvent::Error(format!("{label}: {error:#}")).encode();
                    return;
                }
            };
            yield ReconcileEvent::Log(format!("{label} {change:?}")).encode();

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
    resource: &mut Resource,
) -> anyhow::Result<ChangeStatus> {
    if let Some(config) = resource.config.as_object_mut() {
        config.remove("type");
    }
    let change = state
        .runtime
        .read(&resource.resource_type, &resource.config)
        .await
        .context("read failed")?;
    if change != ChangeStatus::Unchanged {
        state
            .runtime
            .apply(&resource.resource_type, &resource.config)
            .await
            .context("apply failed")?;
    }
    snapshot.insert(resource);
    snapshot.save(&state.storage, key).await.context("Could not save snapshot")?;
    Ok(change)
}
