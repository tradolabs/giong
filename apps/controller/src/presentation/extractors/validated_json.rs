use axum::{
    Json,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub(in crate::presentation) struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(input) =
            Json::<T>::from_request(request, state).await.map_err(IntoResponse::into_response)?;
        input.validate().map_err(|errors| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "code": "http.bad_request", "errors": errors })),
            )
                .into_response()
        })?;
        Ok(Self(input))
    }
}
