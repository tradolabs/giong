use crate::giong::node::secrets;
use crate::types::KubectlProperties;
use giong_node::types::tools::execution::Request;
use giong_node::types::tools::secrets::SecretValue;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

pub(crate) struct DeploymentConfiguration {
    source: String,
    chart: String,
    relative_chart: String,
    release: String,
    namespace: String,
    values_file: String,
    env: BTreeMap<String, String>,
}

impl DeploymentConfiguration {
    pub(crate) fn new(
        properties: KubectlProperties,
        repository_path: &str,
    ) -> Result<Self, String> {
        let path = properties.path.as_deref().unwrap_or(".");
        if Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err("Chart path must be relative without parent traversal".into());
        }

        let chart = Path::new(repository_path).join(path);
        let relative_chart: PathBuf = chart
            .strip_prefix(repository_path)
            .map_err(|_| "Source path is outside its repository")?
            .components()
            .filter(|part| *part != Component::CurDir)
            .collect();
        let relative_chart = if relative_chart.as_os_str().is_empty() {
            ".".into()
        } else {
            relative_chart.to_str().ok_or("Chart path is not UTF-8")?.to_owned()
        };
        let chart = chart.to_str().ok_or("Chart path is not UTF-8")?.to_owned();

        let mut env = properties.env;
        if let Some(kubeconfig) = properties.kubeconfig {
            let value = match kubeconfig {
                SecretValue::Literal(value) => value,
                SecretValue::Reference { secret } => match secret.vault() {
                    Some(vault) => secrets::get_from_vault(vault, secret.key())?,
                    None => secrets::get(secret.key())?,
                },
            };
            env.insert("KUBECONFIG".into(), value);
        }

        Ok(Self {
            source: properties.source,
            chart,
            relative_chart,
            release: properties.release.unwrap_or_else(|| "release".into()),
            namespace: properties.namespace.unwrap_or_else(|| "default".into()),
            values_file: properties.values_file.unwrap_or_default(),
            env,
        })
    }

    pub(crate) fn fingerprint(&self) -> Result<String, String> {
        // Keep checkout locations out of the fingerprint: they change between runs.
        // Resolved credentials are included so rotation triggers a new deployment.
        let bytes = serde_json::to_vec(&json!({
            "source": self.source,
            "path": self.relative_chart,
            "release": self.release,
            "namespace": self.namespace,
            "values_file": self.values_file,
            "env": self.env,
            "dockerfile": include_str!("../Dockerfile"),
            "version": env!("CARGO_PKG_VERSION"),
        }))
        .map_err(|_| "Could not fingerprint deployment configuration")?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub(crate) fn comparison_paths(&self) -> Result<Vec<String>, String> {
        let mut paths = vec![self.relative_chart.clone()];
        if self.values_file.is_empty() {
            return Ok(paths);
        }

        // Values files may be outside the chart, but must stay inside the repository.
        let mut path = PathBuf::new();
        for component in Path::new(&self.relative_chart).join(&self.values_file).components() {
            match component {
                Component::CurDir => {}
                Component::Normal(part) => path.push(part),
                Component::ParentDir if path.pop() => {}
                _ => return Err("Values file must stay within the source repository".into()),
            }
        }
        paths.push(path.to_str().ok_or("Values file path is not UTF-8")?.to_owned());
        Ok(paths)
    }

    pub(crate) fn into_request(self) -> Request {
        Request {
            dockerfile: include_str!("../Dockerfile").into(),
            dockerignore: Some("**/.git\n".into()),
            env: self.env,
            args: vec![self.release, self.namespace, self.values_file, self.chart],
        }
    }
}
