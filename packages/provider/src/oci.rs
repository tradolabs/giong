use crate::error::Error;
use oci_client::client::{Client, ClientConfig};
use oci_client::secrets::RegistryAuth;
use std::{collections::BTreeMap, path::Path, time::Duration};
use wasmtime::Engine;
use wasmtime::component::Component;

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

async fn download(reference: &str) -> Result<BTreeMap<String, Vec<u8>>, Error> {
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
    if image.layers.is_empty() {
        return Err(Error::unexpected(format!("No Wasm layers in {reference}")));
    }

    let mut files = BTreeMap::new();
    for layer in image.layers {
        let name = layer
            .annotations
            .as_ref()
            .and_then(|annotations| annotations.get("org.opencontainers.image.title"))
            .ok_or_else(|| {
                Error::unexpected(format!("Missing Wasm layer filename in {reference}"))
            })?;
        let resource = name
            .strip_suffix(".wasm")
            .filter(|resource| valid_name(resource))
            .ok_or_else(|| {
                Error::unexpected(format!("Invalid Wasm layer filename {name:?} in {reference}"))
            })?;
        if files.insert(resource.to_owned(), layer.data.to_vec()).is_some() {
            return Err(Error::unexpected(format!(
                "Duplicate Wasm resource {resource:?} in {reference}"
            )));
        }
    }
    Ok(files)
}

pub(crate) async fn load(
    engine: &Engine,
    directory: &Path,
    registry: &str,
    version: &str,
    package: &str,
) -> Result<Component, Error> {
    let (provider, resource) =
        package.split_once('/').ok_or_else(|| Error::InvalidResourceType(package.into()))?;
    if !valid_name(provider) || !valid_name(resource) {
        return Err(Error::InvalidResourceType(package.into()));
    }

    // Cache a complete package separately from the former per-resource downloads.
    let parent = directory.join(provider).join(version);
    let cache = parent.join("package");
    let path = cache.join(format!("{resource}.wasm"));
    let compile = |bytes: &[u8]| {
        Component::from_binary(engine, bytes).map_err(|error| {
            Error::unexpected(format!("Could not compile {}: {error}", path.display()))
        })
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) => return compile(&bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Error::unexpected(error)),
    }
    if tokio::fs::try_exists(&cache).await.map_err(Error::unexpected)? {
        return Err(Error::unexpected(format!(
            "Resource {resource:?} is not in cached package {provider}:{version}"
        )));
    }

    let reference = format!("{registry}/{provider}:{version}");
    let files = download(&reference).await?;
    let bytes = files.get(resource).ok_or_else(|| {
        Error::unexpected(format!("Resource {resource:?} is not in package {reference}"))
    })?;
    let component = compile(bytes)?;

    tokio::fs::create_dir_all(&parent).await.map_err(Error::unexpected)?;
    let temporary = tempfile::tempdir_in(&parent).map_err(Error::unexpected)?;
    for (name, bytes) in files {
        tokio::fs::write(temporary.path().join(format!("{name}.wasm")), bytes)
            .await
            .map_err(Error::unexpected)?;
    }
    // Publish all files together so another invocation never sees a partial package.
    match tokio::fs::rename(temporary.path(), &cache).await {
        Ok(()) => Ok(component),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            // Another process finished caching the package first.
            compile(&tokio::fs::read(&path).await.map_err(Error::unexpected)?)
        }
        Err(error) => Err(Error::unexpected(error)),
    }
}
