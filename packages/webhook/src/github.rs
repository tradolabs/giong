use hmac::{Hmac, KeyInit, Mac};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use sha2::Sha256;

pub enum GithubWebhookEvent {
    Ping,
    Push(GithubPushEvent),
    Ignored,
}

#[derive(Deserialize)]
pub struct GithubPushEvent {
    #[serde(rename = "ref")]
    pub reference: String,
    pub deleted: bool,
    pub repository: Repository,
}

#[derive(Deserialize)]
pub struct Repository {
    pub clone_url: String,
}

pub fn parse(
    secret: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<GithubWebhookEvent, (StatusCode, &'static str)> {
    verify_signature(secret, headers, body)?;

    match headers.get("x-github-event").and_then(|value| value.to_str().ok()) {
        Some("ping") => return Ok(GithubWebhookEvent::Ping),
        Some("push") => {}
        Some(_) => return Ok(GithubWebhookEvent::Ignored),
        None => return Err((StatusCode::BAD_REQUEST, "Missing GitHub event\n")),
    }

    let payload: GithubPushEvent = serde_json::from_slice(body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid GitHub push payload\n"))?;
    let Some(branch) = payload.reference.strip_prefix("refs/heads/") else {
        return Ok(GithubWebhookEvent::Ignored);
    };
    if payload.deleted {
        return Ok(GithubWebhookEvent::Ignored);
    }
    if branch.is_empty() || payload.repository.clone_url.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing repository or branch\n"));
    }

    Ok(GithubWebhookEvent::Push(payload))
}

fn verify_signature(
    secret: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), (StatusCode, &'static str)> {
    let invalid_signature = (StatusCode::UNAUTHORIZED, "Invalid webhook signature\n");
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("sha256="))
        .ok_or(invalid_signature)?;
    let mut digest = [0_u8; 32];
    hex::decode_to_slice(signature, &mut digest).map_err(|_| invalid_signature)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Could not verify webhook\n"))?;
    mac.update(body);
    mac.verify_slice(&digest).map_err(|_| invalid_signature)?;

    Ok(())
}
