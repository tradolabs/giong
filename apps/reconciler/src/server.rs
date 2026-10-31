use crate::routes::reconcile;
use axum::{Router, routing::post};
use giong_provider::runtime::Runtime;
use giong_storage::storage::Storage;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) runtime: Arc<Runtime>,
    pub(crate) storage: Arc<Storage>,
}

pub fn router(runtime: Arc<Runtime>, storage: Arc<Storage>) -> Router {
    Router::new()
        .route("/reconcile", post(reconcile::reconcile))
        .with_state(AppState { runtime, storage })
}
