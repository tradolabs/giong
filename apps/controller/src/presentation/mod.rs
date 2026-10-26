use self::controllers::{auth as auth_controller, index, logs, webhook};
use crate::errors::controller_error::ControllerError;
use axum::{
    Router,
    extract::{DefaultBodyLimit, FromRef},
    middleware,
    routing::{get, post},
};
use giong_core::application::application::Application;
use giong_scheduler::Scheduler;
use std::sync::Arc;
use tokio::net::TcpListener;

mod controllers;
mod exceptions;
mod middlewares;
mod schedulers;

#[derive(Clone)]
struct AppState {
    application: Arc<Application>,
    webhook: Arc<webhook::WebhookState>,
}

impl FromRef<AppState> for Arc<Application> {
    fn from_ref(state: &AppState) -> Self {
        state.application.clone()
    }
}

impl FromRef<AppState> for Arc<webhook::WebhookState> {
    fn from_ref(state: &AppState) -> Self {
        state.webhook.clone()
    }
}

pub async fn serve(application: Arc<Application>) -> Result<(), ControllerError> {
    application.config().jwt_secret.as_ref().ok_or(ControllerError::MissingJwtSecret)?;
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    let (webhook_state, webhook_callback) = webhook::setup(application.clone());
    let state = AppState { application: application.clone(), webhook: Arc::new(webhook_state) };

    let router = Router::new()
        .route("/", get(index::index))
        .route("/locales/{locale}", get(index::get_locale))
        .route("/webhooks/github", post(webhook::on_github_event))
        .route("/auth/sign-in", post(auth_controller::sign_in))
        .route(
            "/auth/jobs/{job_id}/logs",
            get(logs::get_logs).route_layer(middleware::from_fn_with_state(
                state.clone(),
                middlewares::auth::require_auth,
            )),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(state);

    eprintln!("HTTP server listening on {}", listener.local_addr()?);

    tokio::try_join!(
        async { axum::serve(listener, router).await.map_err(ControllerError::from) },
        webhook_callback,
        run_scheduler(&application),
    )?;
    Ok(())
}

async fn run_scheduler(application: &Application) -> Result<(), ControllerError> {
    if std::env::var("GIONG_SCHEDULE_ENABLED").as_deref() != Ok("true") {
        return Ok(());
    }

    let mut scheduler = Scheduler::new().await?;
    scheduler
        .register("*/30 * * * * *", schedulers::poll_repositories::handler(application))
        .await?;

    scheduler.start().await?;
    Ok(())
}
