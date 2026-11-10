use crate::presentation::dtos::requests::create_repository::CreateRepositoryRequestDto;
use crate::presentation::dtos::responses::repository::RepositoryResponseDto;
use crate::presentation::exceptions::http_exception::HttpException;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use giong_core::application::application::Application;
use giong_core::application::use_cases::create_repository::CreateRepositoryInput;
use std::sync::Arc;

pub(in crate::presentation) async fn get_repositories(
    State(app): State<Arc<Application>>,
) -> Result<impl IntoResponse, HttpException> {
    Ok(Json(
        app.get_repositories()
            .await?
            .into_iter()
            .map(RepositoryResponseDto::from)
            .collect::<Vec<_>>(),
    ))
}

pub(in crate::presentation) async fn create_repository(
    State(app): State<Arc<Application>>,
    Json(input): Json<CreateRepositoryRequestDto>,
) -> Result<impl IntoResponse, HttpException> {
    let repository = app
        .create_repository(CreateRepositoryInput {
            name: input.name.trim().to_owned(),
            url: input.url.trim().to_owned(),
            token: input.token,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(RepositoryResponseDto::from(repository))))
}
