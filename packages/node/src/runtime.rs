use crate::tools::Tools;
use crate::tools::execution::Execution;
use crate::tools::nodes::Outputs;
use crate::tools::storage::Storage;
use crate::types::metadata::Metadata;
use anyhow::{Context, bail};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

pub(crate) mod bindings {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "node",
    });
}

static ENGINE: LazyLock<Result<Engine, String>> = LazyLock::new(|| {
    let mut config = wasmtime::Config::new();
    config.consume_fuel(true);
    config.wasm_component_model(true);
    Engine::new(&config).map_err(|error| error.to_string())
});

struct CachedComponent {
    bytes: Vec<u8>,
    component: Component,
}

static COMPONENTS: LazyLock<Mutex<HashMap<PathBuf, CachedComponent>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) struct Host {
    pub(crate) events: Option<crate::tools::events::Events>,
    pub(crate) secrets: Option<crate::tools::secrets::Secrets>,
    pub(crate) scope: Option<String>,
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
    pub(crate) workspace: Option<PathBuf>,
    pub(crate) storage: Option<Storage>,
    pub(crate) execution: Option<Execution>,
    pub(crate) outputs: Option<Outputs>,
}

impl WasiView for Host {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.wasi, table: &mut self.table }
    }
}

pub struct Plugin {
    store: Store<Host>,
    node: bindings::Node,
}

impl Plugin {
    pub fn load(kind: &str, tools: Tools) -> anyhow::Result<Self> {
        Self::from_path(path(kind)?, tools)
    }

    pub fn from_path(path: PathBuf, tools: Tools) -> anyhow::Result<Self> {
        let engine = ENGINE.as_ref().map_err(|error| anyhow::anyhow!("{error}"))?;
        let path = std::path::absolute(path)?;
        let bytes = fs::read(&path)
            .with_context(|| format!("Could not read node plugin {}", path.display()))?;
        let component = {
            let mut cache =
                COMPONENTS.lock().map_err(|_| anyhow::anyhow!("Node plugin cache lock failed"))?;
            match cache.get(&path) {
                Some(cached) if cached.bytes == bytes => cached.component.clone(),
                _ => {
                    let component = Component::from_binary(engine, &bytes)
                        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
                    cache.insert(path, CachedComponent { bytes, component: component.clone() });
                    component
                }
            }
        };
        let wasi = WasiCtxBuilder::new()
            .allow_ip_name_lookup(false)
            .allow_tcp(false)
            .allow_udp(false)
            .build();
        let mut store = Store::new(
            engine,
            Host {
                events: tools.events,
                secrets: tools.secrets,
                scope: tools.scope,
                wasi,
                table: ResourceTable::new(),
                limits: StoreLimitsBuilder::new().memory_size(64 * 1024 * 1024).build(),
                workspace: tools.workspace,
                storage: tools.storage,
                execution: tools.execution,
                outputs: tools.outputs,
            },
        );
        store.limiter(|host| &mut host.limits);
        store.set_fuel(50_000_000).map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
            .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        bindings::giong::node::git::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |host| host,
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        bindings::giong::node::storage::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |host| host,
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        bindings::giong::node::execution::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |host| host,
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        bindings::giong::node::secrets::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |host| host,
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        bindings::giong::node::events::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |host| host,
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        bindings::giong::node::nodes::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |host| host,
        )
        .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let node = bindings::Node::instantiate(&mut store, &component, &linker)
            .map_err(|error| anyhow::anyhow!("{error:#}"))
            .context("Plugin does not implement giong:node/node@0.1.0")?;
        Ok(Self { store, node })
    }

    pub fn metadata(&mut self) -> anyhow::Result<Metadata> {
        let json = self
            .node
            .call_metadata(&mut self.store)
            .map_err(|error| anyhow::anyhow!("{error:#}"))?
            .map_err(anyhow::Error::msg)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn configure(&mut self, args: &str) -> anyhow::Result<()> {
        self.node
            .call_configure(&mut self.store, args)
            .map_err(|error| anyhow::anyhow!("{error:#}"))?
            .map_err(anyhow::Error::msg)
    }
}

pub fn path(kind: &str) -> anyhow::Result<PathBuf> {
    if kind.is_empty()
        || kind.split('/').any(|part| {
            part.is_empty()
                || !part.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        })
    {
        bail!("Invalid node plugin kind: {kind}");
    }
    let root = match std::env::var_os("GIONG_PLUGINS_DIR").filter(|path| !path.is_empty()) {
        Some(root) => PathBuf::from(root),
        None => PathBuf::from(
            std::env::var_os("HOME").context("HOME is not set; configure GIONG_PLUGINS_DIR")?,
        )
        .join(".giong/plugins"),
    };
    Ok(root.join(format!("{kind}.wasm")))
}
