use self::controllers::{auth as auth_controller, flows, index, node_runs, vaults};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};
use giong_core::application::application::Application;
use std::sync::Arc;
use tokio::net::TcpListener;

mod controllers;
mod dtos;
mod exceptions;
mod middlewares;

pub async fn serve(application: Arc<Application>) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    let state = application;
    let router = Router::new()
        .route("/", get(index::index))
        .route("/locales/{locale}", get(index::get_locale))
        .route("/auth/sign-in", post(auth_controller::sign_in))
        .merge(
            Router::new()
                .route("/vaults", get(vaults::list).post(vaults::create))
                .route("/vaults/{id}", axum::routing::put(vaults::update).delete(vaults::delete))
                .route("/vaults/{id}/secrets", get(vaults::secrets))
                .route(
                    "/vaults/{id}/secrets/{key}",
                    axum::routing::put(vaults::put_secret).delete(vaults::delete_secret),
                )
                .route("/flows/run", post(flows::run))
                .route("/flows", get(flows::get_flows).post(flows::create_flow))
                .route(
                    "/flows/{flow_id}",
                    axum::routing::put(flows::update_flow).delete(flows::delete_flow),
                )
                .route("/flows/{flow_id}/vaults", axum::routing::put(flows::update_vaults))
                .route("/flows/{flow_id}/nodes/{node_id}/run", post(flows::run_node))
                .route("/flows/{flow_id}/runs", get(flows::runs))
                .route("/runs/{run_id}/logs", get(node_runs::logs))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    middlewares::auth::require_auth,
                )),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middlewares::global::cors()?)
        .with_state(state);

    eprintln!("HTTP server listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;
    Ok(())
}
