use crate::routes::reconcile;
use axum::{Router, routing::post};
use giong_provider::runtime::Runtime;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) runtime: Arc<Runtime>,
}

pub fn router(runtime: Arc<Runtime>) -> Router {
    Router::new()
        .route("/reconcile", post(reconcile::reconcile))
        .with_state(AppState { runtime })
}
