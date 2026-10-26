use crate::presentation::exceptions::http_exception::HttpException;
use crate::presentation::exceptions::http_exception::UnauthorizedException;
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use giong_core::application::application::Application;
use giong_core::application::use_cases::sign_in::Claims;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use std::sync::Arc;

pub(in crate::presentation) async fn require_auth(
    State(application): State<Arc<Application>>,
    request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
        .map(|(_, token)| token);

    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    validation.sub = Some("admin".to_owned());
    validation.set_required_spec_claims(&["exp", "sub"]);

    let Some(secret) = application.config().jwt_secret.as_deref() else {
        return HttpException::InternalServerError("JWT secret is not configured".into())
            .into_response();
    };
    let key = DecodingKey::from_secret(secret.as_bytes());
    if token.is_some_and(|token| decode::<Claims>(token, &key, &validation).is_ok()) {
        next.run(request).await
    } else {
        ([(header::WWW_AUTHENTICATE, "Bearer")], UnauthorizedException("modules.auth.unauthorized"))
            .into_response()
    }
}
