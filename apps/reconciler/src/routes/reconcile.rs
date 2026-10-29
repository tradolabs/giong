use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use giong_provider::{error::Error, protocol::Resource};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
pub(crate) struct ReconcileRequest {
    resources: Vec<Resource>,
}

#[derive(Default, Serialize)]
pub(crate) struct ReconcileResponse {
    created: Vec<Resource>,
    unchanged: Vec<Resource>,
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
        let mut operation = "read";
        eprintln!(
            "Reconciling resource {:?} ({}) operation=read",
            resource.name, resource.resource_type
        );
        let created = tokio::time::timeout(Duration::from_secs(60), async {
            let has_resource =
                state.runtime.read(&resource.resource_type, &resource.config).await?.is_none();
            if has_resource {
                operation = "apply";
                eprintln!(
                    "Reconciling resource {:?} ({}) operation=apply",
                    resource.name, resource.resource_type
                );
                state.runtime.apply(&resource.resource_type, &resource.config, None).await?;
            }
            Ok::<_, Error>(has_resource)
        })
        .await
        .map_err(|_| {
            eprintln!(
                "Resource {:?} ({}) operation={operation} timed out",
                resource.name, resource.resource_type
            );
            (StatusCode::GATEWAY_TIMEOUT, "Provider reconcile timed out".into())
        })?
        .map_err(|error| {
            eprintln!(
                "Resource {:?} ({}) operation={operation} failed: {error}",
                resource.name, resource.resource_type
            );
            (StatusCode::BAD_GATEWAY, error.to_string())
        })?;
        eprintln!(
            "Resource {:?} ({}) {}",
            resource.name,
            resource.resource_type,
            if created { "created" } else { "unchanged" }
        );
        if created {
            response.created.push(resource);
        } else {
            response.unchanged.push(resource);
        }
    }
    Ok(Json(response))
}
