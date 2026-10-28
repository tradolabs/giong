use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use giong_provider::error::Error;
use giong_provider::protocol;
use giong_provider::runtime::Runtime;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

#[derive(Deserialize)]
struct ReconcileRequest {
    resources: Vec<protocol::Resource>,
}

#[derive(Default, Serialize)]
struct ReconcileResponse {
    created: Vec<Resource>,
    unchanged: Vec<Resource>,
}

#[derive(Serialize)]
struct Resource {
    #[serde(rename = "type")]
    resource_type: String,
    name: String,
}

pub fn router(runtime: Arc<Runtime>) -> Router {
    Router::new().route("/reconcile", post(reconcile)).with_state(runtime)
}

async fn reconcile(
    State(runtime): State<Arc<Runtime>>,
    Json(input): Json<ReconcileRequest>,
) -> Result<Json<ReconcileResponse>, (StatusCode, String)> {
    let mut response = ReconcileResponse::default();
    for resource in input.resources {
        let created = tokio::time::timeout(Duration::from_secs(60), async {
            let has_resource =
                runtime.read(&resource.resource_type, &resource.config).await?.is_none();
            if has_resource {
                runtime.apply(&resource.resource_type, &resource.config, None).await?;
            }
            Ok::<_, Error>(has_resource)
        })
        .await
        .map_err(|_| (StatusCode::GATEWAY_TIMEOUT, "Provider reconcile timed out".into()))?
        .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;
        let entry = Resource {
            name: resource
                .config
                .get("name")
                .and_then(|name| name.as_str())
                .unwrap_or(&resource.key)
                .to_owned(),
            resource_type: resource.resource_type,
        };
        if created {
            response.created.push(entry);
        } else {
            response.unchanged.push(entry);
        }
    }
    Ok(Json(response))
}
