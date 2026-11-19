use crate::host::{NodeHost, NodeOutput, NodeScope};
use crate::protocol::{
    Call, HostMessage, MAX_FRAME_BYTES, Operation, PluginMessage, VERSION, decode_frame,
    encode_frame,
};
use crate::sdk::WORKSPACE_ENV;
use crate::types::metadata::Metadata;
use crate::types::node::OUTPUT_KEY;
use anyhow::{Context, anyhow, ensure};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::time::timeout;
use uuid::Uuid;

const IO_TIMEOUT: Duration = Duration::from_secs(120);
const EXIT_TIMEOUT: Duration = Duration::from_secs(10);
const EXECUTION_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const WORKSPACE_DIR: &str = "/workspace";
const PLUGIN_EXECUTABLE: &str = "/usr/local/bin/giong-plugin";

struct Host {
    context: ExecutionContext,
    logger: File,
    outputs: BTreeMap<String, NodeOutput>,
}

pub struct ExecutionContext {
    pub host: Arc<dyn NodeHost>,
    pub scope: NodeScope,
}

impl Host {
    fn new(context: ExecutionContext) -> anyhow::Result<Self> {
        let logger = context.host.open_log(&context.scope)?;
        let outputs = context.host.outputs(&context.scope)?;
        Ok(Self { context, logger, outputs })
    }

    fn dispatch(&mut self, call: Call) -> Result<Value, String> {
        let ExecutionContext { host, scope } = &self.context;
        match call {
            Call::GetSecret { vault, key } => serialize(
                host.get_secret(scope, vault.as_deref(), &key)
                    .map_err(|_| "Secret lookup failed for this flow")?,
            ),
            Call::EmitChanged => {
                host.emit_changed(scope).map_err(|_| "Could not publish node event")?;
                Ok(Value::Null)
            }
            Call::SynchronizeFlow { definitions } => serialize(
                host.synchronize_flow(scope, definitions)
                    .map_err(|_| "Could not synchronize flows")?,
            ),
            Call::GetScope => serialize(scope.flow_id.to_string()),
            Call::GetOutput { name } => self
                .outputs
                .get(&name)
                .map(|output| output.value.clone())
                .ok_or_else(|| "Node has not run or is not part of this flow".into()),
            Call::GetSourceWorkspace { name } => {
                let output =
                    self.outputs.get(&name).ok_or("Source node has no output in this flow")?;
                serialize(node_workspace(scope.flow_id, output.node_id))
            }
            Call::GetStorage { key } => {
                serialize(host.storage(scope).get(&key).map_err(|_| "Storage read failed")?)
            }
            Call::PutStorage { key, value } => {
                host.storage(scope).put(&key, &value).map_err(|_| "Storage write failed")?;
                Ok(Value::Null)
            }
            Call::DeleteStorage { key } => {
                serialize(host.storage(scope).delete(&key).map_err(|_| "Storage delete failed")?)
            }
            Call::SearchStorage { prefix } => {
                serialize(host.storage(scope).search(&prefix).map_err(|_| "Storage search failed")?)
            }
        }
    }
}

fn serialize(value: impl Serialize) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|_| "Could not encode host response".into())
}

pub struct Plugin {
    image: String,
}

impl Plugin {
    pub fn new(kind: &str) -> anyhow::Result<Self> {
        validate_plugin_kind(kind)?;
        Ok(Self { image: image_reference(kind) })
    }

    pub async fn metadata(&self) -> anyhow::Result<Metadata> {
        Ok(serde_json::from_value(self.invoke(Operation::Metadata, None).await?)?)
    }

    pub async fn execute(
        &self,
        context: ExecutionContext,
        properties: Value,
    ) -> anyhow::Result<()> {
        let host = tokio::task::spawn_blocking(move || Host::new(context)).await??;
        self.invoke(Operation::Execute { properties }, Some(host)).await?;
        Ok(())
    }

    async fn invoke(&self, operation: Operation, state: Option<Host>) -> anyhow::Result<Value> {
        let mut command = workspace_container()?;
        let name = format!("giong-plugin-{}", Uuid::now_v7());
        let network = std::env::var("GIONG_PLUGIN_NETWORK").unwrap_or_else(|_| "bridge".into());
        command
            .args(["--name", &name, "--interactive", "--network", &network])
            .args(["--add-host", "host.docker.internal:host-gateway"]);
        let stderr = match &state {
            Some(state) => {
                let scope = &state.context.scope;
                let workspace = node_workspace(scope.flow_id, scope.node_id);
                command.arg("--env").arg(format!("{WORKSPACE_ENV}={workspace}"));
                Stdio::from(state.logger.try_clone()?)
            }
            None => Stdio::inherit(),
        };
        let mut container = RunningContainer { name: Some(name) };
        let mut child = command
            .args(["--entrypoint", "timeout", "--", &self.image, "--kill-after=5s"])
            .arg(format!("{}s", EXECUTION_TIMEOUT.as_secs()))
            .arg(PLUGIN_EXECUTABLE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("Could not start plugin image {}", self.image))?;

        let result = async {
            let output = exchange(&mut child, operation, state, EXECUTION_TIMEOUT).await?;
            let status =
                timeout(EXIT_TIMEOUT, child.wait()).await.context("Plugin did not exit")??;
            container.name = None;
            ensure!(status.success(), "Plugin process exited with {status}");
            output.map_err(anyhow::Error::msg)
        }
        .await;
        drop(child);
        if let Some(name) = container.name.as_deref() {
            remove_container(name).await;
            container.name = None;
        }
        result.with_context(|| format!("Plugin image {}", self.image))
    }
}

struct RunningContainer {
    name: Option<String>,
}

impl Drop for RunningContainer {
    fn drop(&mut self) {
        if let Some(name) = self.name.take()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            runtime.spawn(async move { remove_container(&name).await });
        }
    }
}

async fn remove_container(name: &str) {
    let _ = timeout(
        EXIT_TIMEOUT,
        Command::new("docker")
            .args(["rm", "--force", name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status(),
    )
    .await;
}

async fn exchange(
    child: &mut Child,
    operation: Operation,
    mut state: Option<Host>,
    response_timeout: Duration,
) -> anyhow::Result<Result<Value, String>> {
    let mut stdin = child.stdin.take().context("Missing plugin stdin")?;
    let mut stdout = BufReader::new(child.stdout.take().context("Missing plugin stdout")?);
    let mut frame = Vec::new();
    match receive(&mut stdout, &mut frame, IO_TIMEOUT).await? {
        PluginMessage::Ready { version: VERSION } => {}
        PluginMessage::Ready { version } => {
            anyhow::bail!("Unsupported plugin protocol version {version}")
        }
        _ => anyhow::bail!("Expected a plugin ready message"),
    }
    if let Some(host) = &state {
        let storage = host.context.host.storage(&host.context.scope);
        tokio::task::spawn_blocking(move || storage.delete(OUTPUT_KEY)).await??;
    }
    let mut outgoing = HostMessage::Start { version: VERSION, operation };

    loop {
        let bytes = encode_frame(&outgoing).map_err(anyhow::Error::msg)?;
        timeout(IO_TIMEOUT, stdin.write_all(&bytes)).await.context("Plugin input timed out")??;

        match receive(&mut stdout, &mut frame, response_timeout).await? {
            PluginMessage::Call { id, call } => {
                let mut host =
                    state.take().ok_or_else(|| anyhow!("Metadata cannot call host operations"))?;
                let (host, result) = tokio::task::spawn_blocking(move || {
                    let reply = host.dispatch(call);
                    (host, reply)
                })
                .await?;
                state = Some(host);
                outgoing = HostMessage::Reply { id, result };
            }
            PluginMessage::Finish { result } => return Ok(result),
            PluginMessage::Ready { .. } => anyhow::bail!("Unexpected plugin ready message"),
        }
    }
}

async fn receive(
    stdout: &mut BufReader<ChildStdout>,
    frame: &mut Vec<u8>,
    response_timeout: Duration,
) -> anyhow::Result<PluginMessage> {
    frame.clear();
    timeout(response_timeout, stdout.take((MAX_FRAME_BYTES + 1) as u64).read_until(b'\n', frame))
        .await
        .context("Plugin response timed out")??;
    decode_frame(frame).map_err(anyhow::Error::msg)
}

pub fn validate_plugin_kind(kind: &str) -> anyhow::Result<()> {
    ensure!(
        kind.split('/').all(|part| {
            !part.is_empty()
                && part.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
        }),
        "Invalid node plugin kind: {kind}"
    );
    Ok(())
}

fn image_prefix() -> String {
    std::env::var("GIONG_PLUGIN_IMAGE_PREFIX").unwrap_or_else(|_| "giong-plugin-".into())
}

fn image_tag() -> String {
    std::env::var("GIONG_PLUGIN_IMAGE_TAG").unwrap_or_else(|_| "latest".into())
}

fn image_reference(kind: &str) -> String {
    format!("{}{kind}:{}", image_prefix(), image_tag())
}

fn node_workspace(flow_id: Uuid, node_id: Uuid) -> String {
    format!("{WORKSPACE_DIR}/{flow_id}/node-{node_id}")
}

fn workspace_container() -> anyhow::Result<Command> {
    let volume = std::env::var("GIONG_PLUGIN_WORKSPACE_VOLUME")
        .unwrap_or_else(|_| "giong_plugin-workspaces".into());
    ensure!(
        volume.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
            && volume.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.')),
        "Invalid GIONG_PLUGIN_WORKSPACE_VOLUME name"
    );
    let mut command = Command::new("docker");
    command
        .args([
            "run",
            "--rm",
            "--pull=never",
            "--init",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges:true",
            "--tmpfs",
            "/tmp",
            "--mount",
        ])
        .arg(format!("type=volume,source={volume},target={WORKSPACE_DIR}"))
        .kill_on_drop(true);
    Ok(command)
}

async fn docker_output(command: &mut Command, action: &str) -> anyhow::Result<String> {
    let output = timeout(IO_TIMEOUT, command.stdin(Stdio::null()).kill_on_drop(true).output())
        .await
        .with_context(|| format!("{action} timed out"))?
        .with_context(|| action.to_owned())?;
    ensure!(
        output.status.success(),
        "{action}: {}",
        String::from_utf8_lossy(&output.stderr).trim(),
    );
    Ok(String::from_utf8(output.stdout)?)
}

pub async fn installed_plugin_kinds() -> anyhow::Result<Vec<String>> {
    let output = docker_output(
        Command::new("docker").args(["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"]),
        "Could not list local plugin images",
    )
    .await?;
    let prefix = image_prefix();
    let suffix = format!(":{}", image_tag());
    let mut kinds: Vec<_> = output
        .lines()
        .filter_map(|image| image.strip_prefix(&prefix)?.strip_suffix(&suffix))
        .filter(|kind| validate_plugin_kind(kind).is_ok())
        .map(str::to_owned)
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    Ok(kinds)
}

async fn remove_workspace_directory(path: &str) -> anyhow::Result<()> {
    let kind = installed_plugin_kinds()
        .await?
        .into_iter()
        .next()
        .context("No local plugin image available to remove the workspace")?;
    let image = image_reference(&kind);
    let mut command = workspace_container()?;
    command
        .args(["--network=none", "--entrypoint", "timeout", "--", &image, "--kill-after=5s"])
        .arg(format!("{}s", IO_TIMEOUT.as_secs() - 5))
        .args(["rm", "-rf", "--", path]);
    docker_output(&mut command, "Could not remove plugin workspace").await?;
    Ok(())
}

pub async fn remove_workspace(flow_id: Uuid, node_id: Uuid) -> anyhow::Result<()> {
    remove_workspace_directory(&node_workspace(flow_id, node_id)).await
}

pub async fn remove_flow_workspace(flow_id: Uuid) -> anyhow::Result<()> {
    remove_workspace_directory(&format!("{WORKSPACE_DIR}/{flow_id}")).await
}
