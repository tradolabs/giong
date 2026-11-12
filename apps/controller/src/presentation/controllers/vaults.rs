use crate::presentation::exceptions::http_exception::HttpException;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::secrets::{
    DeleteSecretUseCase, GetSecretsUseCase, PutSecretInput, PutSecretUseCase,
};
use giong_core::application::use_cases::vaults::{
    CreateVaultInput, CreateVaultUseCase, DeleteVaultUseCase, GetVaultsUseCase, UpdateVaultInput,
    UpdateVaultUseCase,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct SecretData {
    value: String,
}
pub(in crate::presentation) async fn list(
    State(app): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(app.execute::<GetVaultsUseCase>(()).await?))
}
pub(in crate::presentation) async fn create(
    State(app): State<Arc<Application>>,
    Json(input): Json<CreateVaultInput>,
) -> Result<impl IntoResponse, HttpException> {
    Ok((StatusCode::CREATED, Json(app.execute::<CreateVaultUseCase>(input).await?)))
}
pub(in crate::presentation) async fn update(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateVaultInput>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(app.execute::<UpdateVaultUseCase>(UpdateVaultInput { id, name: input.name }).await?))
}
pub(in crate::presentation) async fn delete(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    app.execute::<DeleteVaultUseCase>(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(in crate::presentation) async fn secrets(
    State(app): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(app.execute::<GetSecretsUseCase>(id).await?))
}
pub(in crate::presentation) async fn put_secret(
    State(app): State<Arc<Application>>,
    Path((vault_id, key)): Path<(Uuid, String)>,
    Json(data): Json<SecretData>,
) -> Result<impl IntoResponse, HttpException> {
    app.execute::<PutSecretUseCase>(PutSecretInput { vault_id, key, value: data.value }).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(in crate::presentation) async fn delete_secret(
    State(app): State<Arc<Application>>,
    Path(input): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, HttpException> {
    app.execute::<DeleteSecretUseCase>(input).await?;
    Ok(StatusCode::NO_CONTENT)
}
