use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use giong_provider::{error::Error, protocol};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub(crate) struct ReconcileRequest {
    resources: Vec<protocol::Resource>,
}

#[derive(Default, Serialize)]
pub(crate) struct ReconcileResponse {
    created: Vec<protocol::Resource>,
    unchanged: Vec<protocol::Resource>,
}

pub(crate) async fn reconcile(
    State(state): State<AppState>,
    Json(input): Json<ReconcileRequest>,
) -> Result<Json<ReconcileResponse>, (StatusCode, String)> {
    let mut response = ReconcileResponse::default();
    for mut resource in input.resources {
        if let Some(config) = resource.config.as_object_mut() {
            config.remove("type");
        }
        let created = tokio::time::timeout(Duration::from_secs(60), async {
            let has_resource = state
                .runtime
                .read(&resource.resource_type, &resource.config)
                .await?
                .is_none();
            if has_resource {
                state
                    .runtime
                    .apply(&resource.resource_type, &resource.config, None)
                    .await?;
            }
            Ok::<_, Error>(has_resource)
        })
        .await
        .map_err(|_| (StatusCode::GATEWAY_TIMEOUT, "Provider reconcile timed out".into()))?
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;
        if created {
            response.created.push(resource);
        } else {
            response.unchanged.push(resource);
        }
    }
    Ok(Json(response))
}
