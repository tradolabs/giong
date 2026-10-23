use anyhow::Result;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use giong_core::Application;
use giong_webhook::github::{self, GithubPushEvent, GithubWebhookEvent};
use std::future::Future;
use tokio::sync::mpsc;

#[derive(Clone)]
pub(super) struct WebhookState {
    github_secret: Option<String>,
    sender: mpsc::Sender<GithubPushEvent>,
}

pub(super) fn setup(
    application: &Application,
) -> (WebhookState, impl Future<Output = Result<()>> + '_) {
    let github_secret = std::env::var("GIONG_GITHUB_WEBHOOK_SECRET")
        .ok()
        .filter(|value| !value.is_empty());

    if github_secret.is_none() {
        eprintln!("GitHub webhooks disabled: set GIONG_GITHUB_WEBHOOK_SECRET");
    }

    let (sender, receiver) = mpsc::channel(100);
    (
        WebhookState { github_secret, sender },
        callback(application, receiver),
    )
}

pub(super) async fn on_github_event(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, &'static str), (StatusCode, &'static str)> {
    let github_secret = state.github_secret.as_deref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Webhook secret is not configured\n",
    ))?;

    let push_event = match github::parse(github_secret, &headers, &body)? {
        GithubWebhookEvent::Ping => return Ok((StatusCode::OK, "pong\n")),
        GithubWebhookEvent::Ignored => return Ok((StatusCode::NO_CONTENT, "")),
        GithubWebhookEvent::Push(push) => push,
    };

    if let Err(_) = state.sender.try_send(push_event) {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Webhook queue is unavailable\n",
        ));
    }

    Ok((StatusCode::ACCEPTED, "Accepted\n"))
}

async fn callback(
    application: &Application,
    mut receiver: mpsc::Receiver<GithubPushEvent>,
) -> Result<()> {
    while let Some(event) = receiver.recv().await {
        match application
            .poll_repositories(Some(vec![event.repository.clone_url.clone()]))
            .await
        {
            Ok(()) => eprintln!(
                "GitHub push: checked stacks for {} ({})",
                event.repository.clone_url, event.reference
            ),
            Err(error) => eprintln!("GitHub webhook trigger failed: {error:#}"),
        }
    }
    Ok(())
}
