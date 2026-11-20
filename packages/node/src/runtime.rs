use crate::host::{NodeHost, NodeOutput, NodeScope};
use crate::protocol::{
    Call, HostMessage, LogLevel, MAX_FRAME_BYTES, Operation, PluginMessage, VERSION, decode_frame,
    encode_frame,
};
use crate::sdk::WORKSPACE_ENV;
use crate::types::metadata::Metadata;
use crate::types::node::OUTPUT_KEY;
use anyhow::{Context, anyhow, ensure};
use giong_docker::images;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;
use std::process::Stdio;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

const IO_TIMEOUT: Duration = Duration::from_secs(120);
const EXIT_TIMEOUT: Duration = Duration::from_secs(10);
const EXECUTION_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const METADATA_TIMEOUT: Duration = Duration::from_secs(20);
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 30;
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
    pub fn from_image(image: String) -> Self {
        Self { image }
    }

    pub async fn metadata(&self) -> anyhow::Result<Metadata> {
        let mut command = plugin_container();
        command.args([
            "--network=none",
            "--workdir=/tmp",
            "--pids-limit=64",
            "--memory=128m",
        ]);
        let name = format!("giong-plugin-{}", Uuid::now_v7());
        command.args(["--name", &name, "--interactive"]);
        let mut container = RunningContainer { name: Some(name) };
        let mut child = command
            .args(["--entrypoint", "timeout", "--", &self.image, "--kill-after=5s"])
            .arg(format!("{}s", METADATA_TIMEOUT.as_secs()))
            .arg(PLUGIN_EXECUTABLE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("Could not start plugin image {}", self.image))?;

        let result =
            run_plugin_process(&mut child, Operation::Metadata, None, METADATA_TIMEOUT, || {
                container.name = None;
            })
            .await;
        drop(child);
        if let Some(name) = container.name.as_deref() {
            remove_container(name).await;
            container.name = None;
        }
        let value = result.with_context(|| format!("Plugin image {}", self.image))?;
        Ok(serde_json::from_value(value)?)
    }

    pub async fn execute(
        &self,
        context: ExecutionContext,
        properties: Value,
    ) -> anyhow::Result<()> {
        let host = tokio::task::spawn_blocking(move || Host::new(context)).await??;
        let network = std::env::var("GIONG_PLUGIN_NETWORK").unwrap_or_else(|_| "bridge".into());
        let scope = &host.context.scope;
        let workspace = node_workspace(scope.flow_id, scope.node_id);
        let stderr = Stdio::from(host.logger.try_clone()?);

        let lease = container_pool().acquire(&self.image, &network).await?;
        let mut command = Command::new("docker");
        command
            .args(["exec", "--interactive", "--env"])
            .arg(format!("{WORKSPACE_ENV}={workspace}"))
            .arg(lease.name())
            .args(["timeout", "--kill-after=5s"])
            .arg(format!("{}s", EXECUTION_TIMEOUT.as_secs()))
            .arg(PLUGIN_EXECUTABLE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .kill_on_drop(true);

        let mut child = command
            .spawn()
            .with_context(|| format!("Could not exec into plugin container {}", lease.name()))?;

        let result = run_plugin_process(
            &mut child,
            Operation::Execute { properties },
            Some(host),
            EXECUTION_TIMEOUT,
            || {},
        )
        .await;
        drop(child);
        lease.release().await;
        result.with_context(|| format!("Plugin image {}", self.image))?;
        Ok(())
    }
}

async fn run_plugin_process(
    child: &mut Child,
    operation: Operation,
    state: Option<Host>,
    execution_timeout: Duration,
    on_exited: impl FnOnce(),
) -> anyhow::Result<Value> {
    let output = exchange(child, operation, state, execution_timeout).await?;
    let status = timeout(EXIT_TIMEOUT, child.wait()).await.context("Plugin did not exit")??;
    on_exited();
    ensure!(status.success(), "Plugin process exited with {status}");
    output.map_err(anyhow::Error::msg)
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct ContainerKey {
    image: String,
    network: String,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ContainerStatus {
    Unstarted,
    Running,
    Stopped,
}

struct WarmState {
    status: ContainerStatus,
    active: usize,
    generation: u64,
}

struct WarmContainer {
    key: ContainerKey,
    name: String,
    state: Mutex<WarmState>,
}

struct ContainerPool {
    containers: Mutex<HashMap<ContainerKey, Arc<WarmContainer>>>,
}

struct ContainerLease {
    container: Arc<WarmContainer>,
    released: bool,
}

fn container_pool() -> &'static ContainerPool {
    static POOL: OnceLock<ContainerPool> = OnceLock::new();
    POOL.get_or_init(|| ContainerPool { containers: Mutex::new(HashMap::new()) })
}

fn idle_timeout() -> Duration {
    let secs = std::env::var("GIONG_PLUGIN_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

impl ContainerPool {
    async fn acquire(&self, image: &str, network: &str) -> anyhow::Result<ContainerLease> {
        let key = ContainerKey { image: image.to_owned(), network: network.to_owned() };
        loop {
            let container = {
                let mut containers = self.containers.lock().await;
                if let Some(existing) = containers.get(&key) {
                    Arc::clone(existing)
                } else {
                    let created = Arc::new(WarmContainer {
                        key: key.clone(),
                        name: format!("giong-plugin-{}", Uuid::now_v7()),
                        state: Mutex::new(WarmState {
                            status: ContainerStatus::Unstarted,
                            active: 0,
                            generation: 0,
                        }),
                    });
                    containers.insert(key.clone(), Arc::clone(&created));
                    created
                }
            };

            let mut state = container.state.lock().await;
            match state.status {
                ContainerStatus::Unstarted => {
                    if let Err(error) = start_detached_container(&container.key, &container.name).await
                    {
                        state.status = ContainerStatus::Stopped;
                        drop(state);
                        self.remove_entry(&container).await;
                        remove_container(&container.name).await;
                        return Err(error);
                    }
                    state.status = ContainerStatus::Running;
                    state.active += 1;
                    state.generation += 1;
                    drop(state);
                    return Ok(ContainerLease { container, released: false });
                }
                ContainerStatus::Running => {
                    state.active += 1;
                    state.generation += 1;
                    drop(state);
                    return Ok(ContainerLease { container, released: false });
                }
                ContainerStatus::Stopped => {
                    drop(state);
                    self.remove_entry(&container).await;
                }
            }
        }
    }

    async fn remove_entry(&self, target: &Arc<WarmContainer>) {
        let mut containers = self.containers.lock().await;
        if let Some(current) = containers.get(&target.key)
            && Arc::ptr_eq(current, target)
        {
            containers.remove(&target.key);
        }
    }
}

impl WarmContainer {
    async fn release(self: Arc<Self>) {
        let generation = {
            let mut state = self.state.lock().await;
            state.active = state.active.saturating_sub(1);
            state.generation += 1;
            if state.active != 0 || state.status != ContainerStatus::Running {
                return;
            }
            state.generation
        };

        let container = Arc::clone(&self);
        tokio::spawn(async move {
            sleep(idle_timeout()).await;
            container.stop_if_idle(generation).await;
        });
    }

    async fn stop_if_idle(self: Arc<Self>, expected_generation: u64) {
        {
            let mut state = self.state.lock().await;
            if state.active != 0
                || state.generation != expected_generation
                || state.status != ContainerStatus::Running
            {
                return;
            }
            state.status = ContainerStatus::Stopped;
        }
        container_pool().remove_entry(&self).await;
        remove_container(&self.name).await;
    }
}

impl ContainerLease {
    fn name(&self) -> &str {
        &self.container.name
    }

    async fn release(mut self) {
        self.released = true;
        Arc::clone(&self.container).release().await;
    }
}

impl Drop for ContainerLease {
    fn drop(&mut self) {
        if self.released {
            return;
        }
        let container = Arc::clone(&self.container);
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                container.release().await;
            });
        }
    }
}

async fn start_detached_container(key: &ContainerKey, name: &str) -> anyhow::Result<()> {
    let guard_secs = idle_timeout().as_secs().saturating_add(5);
    let script = format!(
        "elapsed=0; while [ \"$elapsed\" -lt {guard_secs} ]; do sleep 5; if pidof giong-plugin >/dev/null 2>&1; then elapsed=0; else elapsed=$((elapsed + 5)); fi; done"
    );
    let mut command = workspace_container()?;
    command.args([
        "--detach",
        "--network",
        &key.network,
        "--add-host",
        "host.docker.internal:host-gateway",
        "--name",
        name,
        "--entrypoint",
        "sh",
        "--",
        &key.image,
        "-c",
        &script,
    ]);
    docker_output(&mut command, &format!("Could not start plugin image {}", key.image)).await?;
    Ok(())
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
    match receive(&mut stdout, &mut frame, response_timeout.min(IO_TIMEOUT)).await? {
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
    send_to_plugin(&mut stdin, HostMessage::Start { version: VERSION, operation }).await?;

    loop {
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
                send_to_plugin(&mut stdin, HostMessage::Reply { id, result }).await?;
            }
            PluginMessage::Log { level, message } => {
                let host = state
                    .as_mut()
                    .ok_or_else(|| anyhow!("Metadata cannot write execution logs"))?;
                let level = match level {
                    LogLevel::Info => "INFO",
                    LogLevel::Error => "ERROR",
                };
                let line = format!("[{level}] {message}\n");
                host.logger.write_all(line.as_bytes()).context("Could not write plugin log")?;
            }
            PluginMessage::Finish { result } => return Ok(result),
            PluginMessage::Ready { .. } => anyhow::bail!("Unexpected plugin ready message"),
        }
    }
}

async fn send_to_plugin(stdin: &mut ChildStdin, message: HostMessage) -> anyhow::Result<()> {
    let bytes = encode_frame(&message).map_err(anyhow::Error::msg)?;
    timeout(IO_TIMEOUT, stdin.write_all(&bytes)).await.context("Plugin input timed out")??;
    Ok(())
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

fn kind_from_image_reference(reference: &str) -> Option<&str> {
    let (repository, tag) = reference.rsplit_once(':')?;
    let kind = repository.strip_prefix(&image_prefix())?;
    (tag == image_tag() && validate_plugin_kind(kind).is_ok()).then_some(kind)
}

fn node_workspace(flow_id: Uuid, node_id: Uuid) -> String {
    format!("{WORKSPACE_DIR}/{flow_id}/node-{node_id}")
}

fn plugin_container() -> Command {
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
            "--user=65532:65532",
            "--tmpfs",
            "/tmp",
        ])
        .kill_on_drop(true);
    command
}

fn workspace_container() -> anyhow::Result<Command> {
    let volume = std::env::var("GIONG_PLUGIN_WORKSPACE_VOLUME")
        .unwrap_or_else(|_| "giong_plugin-workspaces".into());
    ensure!(
        volume.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
            && volume.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.')),
        "Invalid GIONG_PLUGIN_WORKSPACE_VOLUME name"
    );
    let mut command = plugin_container();
    command
        .args(["--workdir", WORKSPACE_DIR, "--mount"])
        .arg(format!("type=volume,source={volume},target={WORKSPACE_DIR}"));
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

async fn remove_workspace_directory(
    path: &str,
    registered_images: &[String],
) -> anyhow::Result<()> {
    let image = images::get_images()
        .await?
        .into_iter()
        .find(|image| {
            kind_from_image_reference(&image.reference).is_some()
                || registered_images.contains(&image.id)
        })
        .context("No local plugin image available to remove the workspace")?;
    let mut command = workspace_container()?;
    command
        .args(["--network=none", "--entrypoint", "timeout", "--", &image.id, "--kill-after=5s"])
        .arg(format!("{}s", IO_TIMEOUT.as_secs() - 5))
        .args(["rm", "-rf", "--", path]);
    docker_output(&mut command, "Could not remove plugin workspace").await?;
    Ok(())
}

pub async fn remove_workspace(
    flow_id: Uuid,
    node_id: Uuid,
    registered_images: &[String],
) -> anyhow::Result<()> {
    remove_workspace_directory(&node_workspace(flow_id, node_id), registered_images).await
}

pub async fn remove_flow_workspace(
    flow_id: Uuid,
    registered_images: &[String],
) -> anyhow::Result<()> {
    remove_workspace_directory(&format!("{WORKSPACE_DIR}/{flow_id}"), registered_images).await
}
