use crate::presentation::dtos::requests::create_secret::CreateSecretRequestDto;
use crate::presentation::dtos::requests::create_vault::CreateVaultRequestDto;
use crate::presentation::dtos::requests::update_secret::UpdateSecretRequestDto;
use crate::presentation::dtos::requests::update_vault::UpdateVaultRequestDto;
use crate::presentation::dtos::responses::secret::SecretResponseDto;
use crate::presentation::dtos::responses::vault::VaultResponseDto;
use crate::presentation::exceptions::http::HttpException;
use crate::presentation::extractors::r#use::Use;
use crate::presentation::extractors::validated_json::ValidatedJson;
use crate::presentation::guards::access_secret::AccessSecretGuard;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::create_vault::CreateVaultInput;
use giong_core::application::use_cases::update_vault::UpdateVaultInput;
use std::sync::Arc;
use uuid::Uuid;
pub(in crate::presentation) async fn get_vaults(
    State(app): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    let vaults = app.get_vaults().await?;
    Ok(Json(vaults.into_iter().map(VaultResponseDto::from).collect::<Vec<_>>()))
}
pub(in crate::presentation) async fn create_vault(
    State(app): State<Arc<Application>>,
    ValidatedJson(input): ValidatedJson<CreateVaultRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let vault = app.create_vault(CreateVaultInput { name: input.name }).await?;
    Ok((StatusCode::CREATED, Json(VaultResponseDto::from(vault))))
}
pub(in crate::presentation) async fn update_vault(
    State(app): State<Arc<Application>>,
    Path(vault_id): Path<Uuid>,
    ValidatedJson(input): ValidatedJson<UpdateVaultRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let vault = app.update_vault(UpdateVaultInput { id: vault_id, name: input.name }).await?;
    Ok(Json(VaultResponseDto::from(vault)))
}
pub(in crate::presentation) async fn delete_vault(
    State(app): State<Arc<Application>>,
    Path(vault_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    app.delete_vault(vault_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(in crate::presentation) async fn get_secrets(
    State(app): State<Arc<Application>>,
    Path(vault_id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpException> {
    let secrets = app.get_secrets(vault_id).await?;
    Ok(Json(secrets.into_iter().map(SecretResponseDto::from).collect::<Vec<_>>()))
}
pub(in crate::presentation) async fn create_secret(
    State(app): State<Arc<Application>>,
    Path(vault_id): Path<Uuid>,
    ValidatedJson(data): ValidatedJson<CreateSecretRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    app.create_secret(vault_id, data.key, data.value).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(in crate::presentation) async fn update_secret(
    _: Use<(AccessSecretGuard,)>,
    State(app): State<Arc<Application>>,
    Path((vault_id, secret_id)): Path<(Uuid, Uuid)>,
    ValidatedJson(data): ValidatedJson<UpdateSecretRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    app.update_secret(vault_id, secret_id, data.value).await?;
    Ok(StatusCode::NO_CONTENT)
}
pub(in crate::presentation) async fn delete_secret(
    _: Use<(AccessSecretGuard,)>,
    State(app): State<Arc<Application>>,
    Path((vault_id, secret_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, HttpException> {
    app.delete_secret(vault_id, secret_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
