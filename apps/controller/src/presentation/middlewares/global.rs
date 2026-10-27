use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};

pub(in crate::presentation) fn cors() -> anyhow::Result<CorsLayer> {
    let origin = std::env::var("GIONG_CORS_ORIGIN").unwrap_or_else(|_| "*".to_owned());
    let origin = origin.trim();
    let cors = if origin.is_empty() || origin == "*" {
        CorsLayer::new().allow_origin(Any)
    } else {
        CorsLayer::new().allow_origin(origin.parse::<HeaderValue>()?)
    };
    Ok(cors.allow_methods(Any).allow_headers(Any))
}
