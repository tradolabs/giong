use crate::error::Error;
use crate::types::{Metadata, Plan};
use anyhow::{Context, bail};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "runner",
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

struct Host {
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
}

impl WasiView for Host {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.wasi, table: &mut self.table }
    }
}

struct Plugin {
    store: Store<Host>,
    runner: bindings::Runner,
}

impl Plugin {
    fn load(id: &str) -> anyhow::Result<Self> {
        // Runtime IDs are filenames, never paths outside the plugin directory.
        if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            bail!("Invalid runner plugin ID: {id}");
        }
        let engine = ENGINE.as_ref().map_err(|error| anyhow::anyhow!("{error}"))?;
        let path = std::path::absolute(directory()?.join(format!("{id}.wasm")))?;
        let bytes =
            fs::read(&path).with_context(|| format!("Could not read runner {}", path.display()))?;
        let component = {
            let mut cache =
                COMPONENTS.lock().map_err(|_| anyhow::anyhow!("Runner cache lock failed"))?;
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
                wasi,
                table: ResourceTable::new(),
                limits: StoreLimitsBuilder::new().memory_size(64 * 1024 * 1024).build(),
            },
        );
        store.limiter(|host| &mut host.limits);
        store.set_fuel(50_000_000).map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)
            .map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let runner = bindings::Runner::instantiate(&mut store, &component, &linker)
            .map_err(|error| anyhow::anyhow!("{error:#}"))
            .context("Runner does not implement giong:runner/runner@0.1.0")?;
        Ok(Self { store, runner })
    }

    fn metadata(&mut self) -> anyhow::Result<Metadata> {
        let metadata = self
            .runner
            .call_metadata(&mut self.store)
            .map_err(|error| anyhow::anyhow!("{error:#}"))?
            .map_err(anyhow::Error::msg)?;
        Ok(Metadata {
            id: metadata.id,
            name: metadata.name,
            version: metadata.version,
            description: metadata.description,
            configuration_schema: serde_json::from_str(&metadata.configuration_schema)?,
            defaults: serde_json::from_str(&metadata.defaults)?,
        })
    }

    fn plan(&mut self, stack_name: &str, configurations: &Value) -> anyhow::Result<Plan> {
        let configurations = serde_json::to_string(configurations)?;
        let plan = self
            .runner
            .call_plan(&mut self.store, stack_name, &configurations)
            .map_err(|error| anyhow::anyhow!("{error:#}"))?
            .map_err(anyhow::Error::msg)?;
        match plan {
            bindings::giong::runner::types::Plan::Docker(plan) => Ok(Plan::Docker {
                dockerfile: plan.dockerfile,
                dockerignore: plan.dockerignore,
                args: plan.args,
            }),
        }
    }
}

fn directory() -> anyhow::Result<PathBuf> {
    let home = || {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set; configure GIONG_RUNNERS_DIR")
    };
    match std::env::var_os("GIONG_RUNNERS_DIR").filter(|path| !path.is_empty()) {
        Some(path) => {
            let path = PathBuf::from(path);
            match path.strip_prefix("~") {
                Ok(relative) => Ok(home()?.join(relative)),
                Err(_) => Ok(path),
            }
        }
        None => Ok(home()?.join(".giong/plugins/runners")),
    }
}

pub fn get_plugins() -> Result<Vec<Metadata>, Error> {
    let result = (|| -> anyhow::Result<Vec<Metadata>> {
        let directory = directory()?;
        let entries = fs::read_dir(&directory)
            .with_context(|| format!("Could not read runner directory {}", directory.display()))?;
        let mut plugins = Vec::new();
        for entry in entries {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("wasm") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|name| name.to_str())
                .context("Invalid runner filename")?;
            plugins.push(Plugin::load(id)?.metadata()?);
        }
        plugins.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(plugins)
    })();
    result.map_err(|error| Error::Plugin(format!("{error:#}")))
}

pub fn plan(stack_name: &str, configurations: &Value) -> Result<Plan, Error> {
    let result = (|| -> anyhow::Result<Plan> {
        let runtime = configurations
            .get("runtime")
            .and_then(Value::as_str)
            .context("Configurations must include a string \"runtime\"")?;
        let settings = configurations
            .get("configuration")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        settings.as_object().context("\"configuration\" must be a TOML table")?;
        Plugin::load(runtime)?.plan(stack_name, &settings)
    })();
    result.map_err(|error| Error::Plugin(format!("{error:#}")))
}
