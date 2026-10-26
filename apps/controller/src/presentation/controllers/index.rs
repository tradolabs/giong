use crate::presentation::exceptions::http_exception::HttpException;
use crate::presentation::exceptions::http_exception::NotFoundException;
use axum::{
    extract::Path,
    http::header,
    response::{Html, IntoResponse},
};

pub(in crate::presentation) async fn index() -> Result<impl IntoResponse, HttpException> {
    Ok(Html(include_str!("../static/index.html")))
}

pub(in crate::presentation) async fn get_locale(
    Path(locale): Path<String>,
) -> Result<impl IntoResponse, HttpException> {
    let content = match locale.as_str() {
        "errors.en.json" => include_str!("../static/locales/errors.en.json"),
        "errors.vi.json" => include_str!("../static/locales/errors.vi.json"),
        _ => return Err(NotFoundException("http.not_found")),
    };
    Ok(([(header::CONTENT_TYPE, "application/json; charset=utf-8")], content))
}
