use crate::presentation::exceptions::http::HttpException;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct InstallPlugin {
    image: String,
    #[serde(default)]
    pull: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct RemovePlugin {
    kind: String,
}

pub(in crate::presentation) async fn get_plugins(
    State(application): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(application.get_plugins().await?))
}

pub(in crate::presentation) async fn install_plugin(
    State(application): State<Arc<Application>>,
    Json(input): Json<InstallPlugin>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(application.install_plugin(&input.image, input.pull).await?))
}

pub(in crate::presentation) async fn remove_plugin(
    State(application): State<Arc<Application>>,
    Query(input): Query<RemovePlugin>,
) -> Result<impl IntoResponse, HttpException> {
    application.remove_plugin(&input.kind).await?;
    Ok(StatusCode::NO_CONTENT)
}
