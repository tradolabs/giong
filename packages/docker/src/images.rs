use crate::constants::{DEFAULT_DOCKER_SOCKET, DOCKER_API_URL, REQUEST_TIMEOUT};
use crate::error::Error;
use crate::types::{DockerImage, DockerMessage, DockerPullProgress, LocalImage};
use reqwest::{Client, Response, StatusCode, Url, redirect::Policy};
use std::io;

#[cfg(unix)]
fn connect_to_docker() -> Result<Client, Error> {
    let host = std::env::var("DOCKER_HOST").unwrap_or_default();
    let socket = if host.is_empty() {
        DEFAULT_DOCKER_SOCKET
    } else {
        host.strip_prefix("unix://").filter(|path| path.starts_with('/')).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "DOCKER_HOST must be an absolute Unix socket URL",
            )
        })?
    };
    Ok(Client::builder()
        .unix_socket(socket)
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::none())
        .build()?)
}

#[cfg(not(unix))]
fn connect_to_docker() -> Result<Client, Error> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Docker image operations require a Unix socket")
        .into())
}

pub async fn get_images() -> Result<Vec<LocalImage>, Error> {
    let response = connect_to_docker()?
        .get(format!("{DOCKER_API_URL}/images/json"))
        .query(&[("all", true)])
        .send()
        .await?;
    let images = parse_response(response).await?.json::<Vec<DockerImage>>().await?;

    let mut local = Vec::with_capacity(images.len());
    for image in images {
        let id = image.id.ok_or(Error::InvalidImageId)?;
        let tags = image.repo_tags.unwrap_or_default();
        if tags.is_empty() {
            local.push(LocalImage { reference: id.clone(), id });
        } else {
            local
                .extend(tags.into_iter().map(|reference| LocalImage { id: id.clone(), reference }));
        }
    }
    Ok(local)
}

pub async fn inspect_image(reference: &str) -> Result<String, Error> {
    let mut url = Url::parse(DOCKER_API_URL).expect("Docker API URL is valid");
    url.path_segments_mut()
        .expect("Docker API URL has a path")
        .extend(["images", reference, "json"]);
    let response = connect_to_docker()?.get(url).send().await?;
    let image = parse_response(response).await?.json::<DockerImage>().await?;
    image
        .id
        .filter(|id| {
            id.strip_prefix("sha256:").is_some_and(|digest| {
                digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
            })
        })
        .ok_or(Error::InvalidImageId)
}

pub async fn pull_image(reference: &str) -> Result<(), Error> {
    let client = connect_to_docker()?;
    let mut request =
        client.post(format!("{DOCKER_API_URL}/images/create")).query(&[("fromImage", reference)]);
    let name = reference.rsplit('/').next().unwrap_or(reference);
    if !reference.contains('@') && !name.contains(':') {
        request = request.query(&[("tag", "latest")]);
    }
    let response = parse_response(request.send().await?).await?;
    let progress = response.bytes().await?;
    for item in serde_json::Deserializer::from_slice(&progress).into_iter::<DockerPullProgress>() {
        let item = item?;
        if let Some(message) = item.error_detail.map(|detail| detail.message).or(item.error) {
            return Err(io::Error::other(message).into());
        }
    }
    Ok(())
}

async fn parse_response(response: Response) -> Result<Response, Error> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await?;
    let message =
        serde_json::from_str::<DockerMessage>(&body).map(|error| error.message).unwrap_or(body);
    let kind = if status == StatusCode::NOT_FOUND {
        io::ErrorKind::NotFound
    } else {
        io::ErrorKind::Other
    };
    Err(io::Error::new(kind, format!("Docker API returned {status}: {message}")).into())
}
