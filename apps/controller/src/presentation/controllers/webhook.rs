use crate::presentation::exceptions::http_exception::HttpException;
use crate::presentation::exceptions::http_exception::UnauthorizedException;
use crate::presentation::exceptions::http_exception::{
    BadRequestException, ServiceUnavailableException,
};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use giong_core::application::application::Application;
use giong_webhook::github::{self, GithubPushEvent, GithubWebhookEvent};
use std::{future::Future, sync::Arc};
use tokio::sync::mpsc;

#[derive(Clone)]
pub(in crate::presentation) struct WebhookState {
    github_secret: Option<String>,
    sender: mpsc::Sender<GithubPushEvent>,
}

pub(in crate::presentation) fn setup(
    application: Arc<Application>,
) -> (WebhookState, impl Future<Output = anyhow::Result<()>>) {
    let github_secret =
        std::env::var("GIONG_GITHUB_WEBHOOK_SECRET").ok().filter(|value| !value.is_empty());

    if github_secret.is_none() {
        eprintln!("GitHub webhooks disabled: set GIONG_GITHUB_WEBHOOK_SECRET");
    }

    let (sender, receiver) = mpsc::channel(100);
    (WebhookState { github_secret, sender }, callback(application, receiver))
}

pub(in crate::presentation) async fn on_github_event(
    State(state): State<Arc<WebhookState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, HttpException> {
    let github_secret = state
        .github_secret
        .as_deref()
        .ok_or(ServiceUnavailableException("modules.webhook.service_unavailable"))?;

    let event = github::parse(github_secret, &headers, &body).map_err(
        |(status, message)| -> HttpException {
            match status {
                StatusCode::UNAUTHORIZED => {
                    UnauthorizedException("modules.webhook.invalid_signature")
                }
                StatusCode::INTERNAL_SERVER_ERROR => {
                    HttpException::InternalServerError(message.to_string())
                }
                _ => BadRequestException("modules.webhook.invalid_payload"),
            }
        },
    )?;

    match event {
        GithubWebhookEvent::Ping => Ok((StatusCode::OK, "pong")),
        GithubWebhookEvent::Ignored => Ok((StatusCode::NO_CONTENT, "")),
        GithubWebhookEvent::Push(push_event) => {
            if state.sender.try_send(push_event).is_err() {
                return Err(ServiceUnavailableException("modules.webhook.queue_unavailable"));
            }
            Ok((StatusCode::ACCEPTED, "Accepted"))
        }
    }
}

async fn callback(
    application: Arc<Application>,
    mut receiver: mpsc::Receiver<GithubPushEvent>,
) -> anyhow::Result<()> {
    while let Some(event) = receiver.recv().await {
        let repository_url = event.repository.clone_url.clone();
        let reference = event.reference.clone();
        match application.poll_repositories(Some(vec![repository_url.clone()])).await {
            Ok(()) => {
                eprintln!("GitHub push: checked stacks for {} ({})", repository_url, reference)
            }
            Err(error) => eprintln!("GitHub webhook trigger failed: {error:#}"),
        }
    }
    Ok(())
}
