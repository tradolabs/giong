use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use std::marker::PhantomData;

pub(in crate::presentation) struct Use<T>(PhantomData<fn() -> T>);

impl<S, T> FromRequestParts<S> for Use<T>
where
    S: Send + Sync,
    T: FromRequestParts<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        T::from_request_parts(parts, state).await.map_err(IntoResponse::into_response)?;
        Ok(Self(PhantomData))
    }
}
