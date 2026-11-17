use self::controllers::{auth as auth_controller, flows, index, node, vault};
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
mod extractors;
mod guards;
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
                .route("/nodes/kinds", get(node::get_kinds))
                .route("/vaults", get(vault::get_vaults).post(vault::create_vault))
                .route(
                    "/vaults/{vault_id}",
                    axum::routing::put(vault::update_vault).delete(vault::delete_vault),
                )
                .route(
                    "/vaults/{vault_id}/secrets",
                    get(vault::get_secrets).post(vault::create_secret),
                )
                .route(
                    "/vaults/{vault_id}/secrets/{secret_id}",
                    axum::routing::put(vault::update_secret).delete(vault::delete_secret),
                )
                .route("/flows", get(flows::get_flows).post(flows::create_flow))
                .route(
                    "/flows/{flow_id}",
                    axum::routing::put(flows::update_flow).delete(flows::delete_flow),
                )
                .route("/flows/{flow_id}/nodes/{node_id}/execute", post(flows::execute_node))
                .route("/flows/{flow_id}/nodes", post(node::create_node))
                .route(
                    "/flows/{flow_id}/nodes/{node_id}",
                    get(node::get_details).put(node::update_node).delete(node::delete_node),
                )
                .route("/logs/{log_id}", get(node::get_logs))
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
