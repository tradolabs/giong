use crate::presentation::dtos::requests::{
    create_secret::CreateSecretRequestDto, update_secret::UpdateSecretRequestDto,
};
use crate::presentation::exceptions::http_exception::HttpException;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub(in crate::presentation) struct CreateSet {
    name: String,
}
#[derive(Deserialize)]
pub(in crate::presentation) struct InheritSets {
    inherited_secret_set_ids: Vec<Uuid>,
}

pub(in crate::presentation) async fn list(
    State(app): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(app.get_secret_sets().await?))
}
pub(in crate::presentation) async fn create(
    State(app): State<Arc<Application>>,
    Json(input): Json<CreateSet>,
) -> Result<impl IntoResponse, HttpException> {
    Ok((StatusCode::CREATED, Json(app.create_secret_set(input.name).await?)))
}
pub(in crate::presentation) async fn delete(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    app.delete_secret_set(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(in crate::presentation) async fn secrets(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(app.get_set_secrets(id).await?))
}
pub(in crate::presentation) async fn create_secret(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateSecretRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    app.create_set_secret(id, input.key, input.value).await?;
    Ok((StatusCode::CREATED, Json(app.get_set_secrets(id).await?)))
}
pub(in crate::presentation) async fn update_secret(
    State(app): State<Arc<Application>>,
    Path((id, key)): Path<(Uuid, String)>,
    Json(input): Json<UpdateSecretRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    app.update_set_secret(id, key, input.value).await?;
    Ok(Json(app.get_set_secrets(id).await?))
}
pub(in crate::presentation) async fn delete_secret(
    State(app): State<Arc<Application>>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, HttpException> {
    app.delete_set_secret(id, key).await?;
    Ok(Json(app.get_set_secrets(id).await?))
}
pub(in crate::presentation) async fn inherit(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
    Json(input): Json<InheritSets>,
) -> Result<impl IntoResponse, HttpException> {
    app.inherit_secret_sets(id, input.inherited_secret_set_ids).await?;
    Ok(Json(app.get_secrets(id).await?))
}
