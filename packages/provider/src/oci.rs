use crate::error::Error;
use oci_client::client::{Client, ClientConfig};
use oci_client::secrets::RegistryAuth;
use std::{path::Path, time::Duration};
use wasmtime::Engine;
use wasmtime::component::Component;

async fn download(reference: &str) -> Result<Vec<u8>, Error> {
    let client = Client::new(ClientConfig {
        connect_timeout: Some(Duration::from_secs(10)),
        read_timeout: Some(Duration::from_secs(30)),
        ..Default::default()
    });
    let image_reference = reference.parse::<oci_client::Reference>().map_err(Error::unexpected)?;
    let image = client
        .pull(&image_reference, &RegistryAuth::Anonymous, vec!["application/wasm"])
        .await
        .map_err(Error::unexpected)?;
    if image.layers.len() != 1 {
        return Err(Error::unexpected(format!("Expected one Wasm layer in {reference}")));
    }
    Ok(image.layers.into_iter().next().unwrap().data.to_vec())
}

pub(crate) async fn load(
    engine: &Engine,
    directory: &Path,
    registry: &str,
    version: &str,
    resource_type: &str,
) -> Result<Component, Error> {
    let (provider, resource) = resource_type
        .split_once('.')
        .ok_or_else(|| Error::InvalidResourceType(resource_type.into()))?;
    for part in [provider, resource] {
        if part.is_empty()
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(Error::InvalidResourceType(resource_type.into()));
        }
    }
    let path = directory.join(provider).join(version).join(format!("{resource}.wasm"));
    let (bytes, downloaded) = match tokio::fs::read(&path).await {
        Ok(bytes) => (bytes, false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let reference = format!("{}/{provider}/{resource}:{}", registry, version);
            (download(&reference).await?, true)
        }
        Err(error) => {
            return Err(Error::unexpected(error));
        }
    };
    let component = Component::from_binary(engine, &bytes).map_err(|error| {
        Error::unexpected(format!("Could not compile {}: {error}", path.display()))
    })?;
    if downloaded {
        let directory =
            path.parent().ok_or_else(|| Error::unexpected("Missing provider cache directory"))?;
        tokio::fs::create_dir_all(directory).await.map_err(Error::unexpected)?;
        let temporary = tempfile::NamedTempFile::new_in(directory).map_err(Error::unexpected)?;
        tokio::fs::write(temporary.path(), &bytes).await.map_err(Error::unexpected)?;
        temporary.persist(&path).map_err(Error::unexpected)?;
    }
    Ok(component)
}
