use crate::host::{Flow, FlowSyncResult};
use crate::protocol::{
    Call, HostMessage, LogLevel, MAX_FRAME_BYTES, Operation, PluginMessage, VERSION, decode_frame,
    encode_frame,
};
pub use crate::types::metadata::Metadata;
use crate::types::node::OUTPUT_KEY;
use crate::types::secrets::SecretValue;
use anyhow::{Context as _, Error, Result, anyhow, bail, ensure};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) const WORKSPACE_ENV: &str = "GIONG_NODE_WORKSPACE";

/// Implement a plugin executable. Use `ctx.log_info` and `ctx.log_err` to write execution logs.
pub trait Plugin {
    type Input: DeserializeOwned;

    fn metadata() -> Metadata;
    fn execute(ctx: &mut Context, input: Self::Input) -> Result<()>;
}

/// Run one plugin invocation using JSON Lines on stdin and stdout.
pub fn run<P: Plugin>() -> ExitCode {
    let mut ctx = Context {
        writer: io::stdout().lock(),
        reader: io::stdin().lock(),
        next_id: 1,
    };
    match serve::<P>(&mut ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn serve<P: Plugin>(ctx: &mut Context) -> Result<()> {
    ctx.send(PluginMessage::Ready { version: VERSION })?;
    let HostMessage::Start { version, operation } = ctx.receive()? else {
        bail!("Expected a plugin start message");
    };
    ensure!(version == VERSION, "Unsupported plugin protocol version {version}");
    let result = match operation {
        Operation::Metadata => {
            serde_json::to_value(P::metadata()).context("Could not encode plugin metadata")
        }
        Operation::Execute { properties } => serde_json::from_value(properties)
            .context("Invalid plugin properties")
            .and_then(|input| P::execute(ctx, input))
            .map(|()| Value::Null),
    };
    ctx.send(PluginMessage::Finish { result: result.map_err(|error| error.to_string()) })
}

pub struct Context {
    writer: io::StdoutLock<'static>,
    reader: io::StdinLock<'static>,
    next_id: u64,
}

impl Context {
    pub fn log_info(&mut self, message: impl Into<String>) -> Result<()> {
        self.send(PluginMessage::Log { level: LogLevel::Info, message: message.into() })
    }

    pub fn log_err(&mut self, message: impl Into<String>) -> Result<()> {
        self.send(PluginMessage::Log { level: LogLevel::Error, message: message.into() })
    }

    fn send(&mut self, message: PluginMessage) -> Result<()> {
        self.writer
            .write_all(&encode_frame(&message).map_err(Error::msg)?)
            .and_then(|()| self.writer.flush())
            .context("Could not write plugin message")
    }

    fn receive(&mut self) -> Result<HostMessage> {
        let mut line = Vec::new();
        self.reader
            .by_ref()
            .take((MAX_FRAME_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)
            .context("Could not read plugin message")?;
        decode_frame(&line).map_err(Error::msg)
    }

    fn call<T: DeserializeOwned>(&mut self, call: Call) -> Result<T> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(PluginMessage::Call { id, call })?;
        match self.receive()? {
            HostMessage::Reply { id: reply_id, result } if reply_id == id => {
                serde_json::from_value(result.map_err(Error::msg)?).context("Invalid host response")
            }
            _ => bail!("Unexpected host response"),
        }
    }

    pub fn get_secret(&mut self, vault: Option<&str>, key: &str) -> Result<String> {
        self.call(Call::GetSecret { vault: vault.map(str::to_owned), key: key.into() })
    }

    pub fn resolve_secret(&mut self, value: impl TryInto<SecretValue>) -> Result<String> {
        match value.try_into().map_err(|_| anyhow!("Invalid secret value"))? {
            SecretValue::Literal(value) => Ok(value),
            SecretValue::Reference { secret } => self.get_secret(secret.vault(), secret.key()),
        }
    }

    pub fn emit_changed(&mut self) -> Result<()> {
        self.call(Call::EmitChanged)
    }

    pub fn synchronize_flow(&mut self, definitions: Vec<Flow>) -> Result<FlowSyncResult> {
        self.call(Call::SynchronizeFlow { definitions })
    }

    /// Open the node's directory in the shared plugin workspace.
    pub fn open_workspace(&self) -> Result<PathBuf> {
        let directory =
            std::env::var_os(WORKSPACE_ENV).context("Plugin workspace is unavailable")?;
        let path = PathBuf::from(directory);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn get_source_workspace(&mut self, name: &str) -> Result<String> {
        self.call(Call::GetSourceWorkspace { name: name.into() })
    }

    pub fn get_scope(&mut self) -> Result<String> {
        self.call(Call::GetScope)
    }

    pub fn get_output<T: DeserializeOwned>(&mut self, name: &str) -> Result<T> {
        self.call(Call::GetOutput { name: name.into() })
    }

    pub fn save_output(&mut self, output: &impl Serialize) -> Result<()> {
        let bytes = serde_json::to_vec(output).context("Could not encode node output")?;
        self.put_storage(OUTPUT_KEY, bytes)
    }

    pub fn get_storage(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        self.call(Call::GetStorage { key: key.into() })
    }

    pub fn put_storage(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.call(Call::PutStorage { key: key.into(), value })
    }

    pub fn delete_storage(&mut self, key: &str) -> Result<bool> {
        self.call(Call::DeleteStorage { key: key.into() })
    }

    pub fn search_storage(&mut self, prefix: &str) -> Result<Vec<String>> {
        self.call(Call::SearchStorage { prefix: prefix.into() })
    }
}
