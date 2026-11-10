use self::controllers::secret_sets;
use self::controllers::{auth as auth_controller, index, jobs, repositories, stacks, webhook};
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
mod dtos;
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

pub async fn serve(application: Arc<Application>) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    let (webhook_state, webhook_callback) = webhook::setup(application.clone());
    let state = AppState { application: application.clone(), webhook: Arc::new(webhook_state) };

    let router = Router::new()
        .route("/", get(index::index))
        .route("/locales/{locale}", get(index::get_locale))
        .route("/webhooks/github", post(webhook::on_github_event))
        .route("/auth/sign-in", post(auth_controller::sign_in))
        .merge(
            Router::new()
                .route("/secret-sets", get(secret_sets::list).post(secret_sets::create))
                .route("/secret-sets/{id}", axum::routing::delete(secret_sets::delete))
                .route(
                    "/secret-sets/{id}/secrets",
                    get(secret_sets::secrets).post(secret_sets::create_secret),
                )
                .route(
                    "/secret-sets/{id}/secrets/{key}",
                    axum::routing::put(secret_sets::update_secret)
                        .delete(secret_sets::delete_secret),
                )
                .route("/stacks/{id}/secret-sets", axum::routing::put(secret_sets::inherit))
                .route("/runners", get(stacks::get_runners))
                .route("/stacks", get(stacks::get_stacks).post(stacks::create_stack))
                .route(
                    "/repositories",
                    get(repositories::get_repositories).post(repositories::create_repository),
                )
                .route("/stacks/{stack_id}", axum::routing::put(stacks::update_stack))
                .route("/stacks/{stack_id}/jobs", get(stacks::get_jobs).post(jobs::create_job))
                .route(
                    "/stacks/{stack_id}/secrets",
                    get(stacks::get_secrets).post(stacks::create_secret),
                )
                .route(
                    "/stacks/{stack_id}/secrets/{key}",
                    axum::routing::put(stacks::update_secret).delete(stacks::delete_secret),
                )
                .route("/jobs/{job_id}/logs", get(jobs::get_job_logs))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    middlewares::auth::require_auth,
                )),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middlewares::global::cors()?)
        .with_state(state);

    eprintln!("HTTP server listening on {}", listener.local_addr()?);

    tokio::try_join!(
        async { axum::serve(listener, router).await.map_err(anyhow::Error::from) },
        webhook_callback,
        schedule(&application),
    )?;

    Ok(())
}

async fn schedule(application: &Application) -> anyhow::Result<()> {
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
