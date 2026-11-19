use crate::host::{Flow, FlowSyncResult};
use crate::protocol::{
    Call, HostMessage, MAX_FRAME_BYTES, Operation, PluginMessage, VERSION, decode_frame,
    encode_frame,
};
pub use crate::types::metadata::Metadata;
use crate::types::node::OUTPUT_KEY;
use crate::types::secrets::SecretValue;
use anyhow::{Context, Error, Result, anyhow, bail, ensure};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) const WORKSPACE_ENV: &str = "GIONG_NODE_WORKSPACE";

/// Implement a plugin executable. Write diagnostic logs to stderr.
pub trait Plugin {
    type Input: DeserializeOwned;

    fn metadata() -> Metadata;
    fn execute(client: &mut Client<'_>, input: Self::Input) -> Result<()>;
}

/// Run one plugin invocation using JSON Lines on stdin and stdout.
pub fn run<P: Plugin>() -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    match serve::<P>(&mut reader, &mut writer) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn serve<P: Plugin>(reader: &mut dyn BufRead, writer: &mut dyn Write) -> Result<()> {
    send(writer, PluginMessage::Ready { version: VERSION })?;
    let HostMessage::Start { version, operation } = receive(reader)? else {
        bail!("Expected a plugin start message");
    };
    ensure!(version == VERSION, "Unsupported plugin protocol version {version}");
    let result = match operation {
        Operation::Metadata => {
            serde_json::to_value(P::metadata()).context("Could not encode plugin metadata")
        }
        Operation::Execute { properties } => serde_json::from_value(properties)
            .context("Invalid plugin properties")
            .and_then(|input| P::execute(&mut Client::attach(reader, writer), input))
            .map(|()| Value::Null),
    };
    send(writer, PluginMessage::Finish { result: result.map_err(|error| error.to_string()) })
}

fn send(writer: &mut dyn Write, message: PluginMessage) -> Result<()> {
    writer
        .write_all(&encode_frame(&message).map_err(Error::msg)?)
        .and_then(|()| writer.flush())
        .context("Could not write plugin message")
}

fn receive(reader: &mut dyn BufRead) -> Result<HostMessage> {
    let mut line = Vec::new();
    reader
        .take((MAX_FRAME_BYTES + 1) as u64)
        .read_until(b'\n', &mut line)
        .context("Could not read plugin message")?;
    decode_frame(&line).map_err(Error::msg)
}

pub struct Client<'a> {
    reader: &'a mut dyn BufRead,
    writer: &'a mut dyn Write,
    next_id: u64,
}

impl<'a> Client<'a> {
    fn attach(reader: &'a mut dyn BufRead, writer: &'a mut dyn Write) -> Self {
        Self { reader, writer, next_id: 1 }
    }

    fn call<T: DeserializeOwned>(&mut self, call: Call) -> Result<T> {
        let id = self.next_id;
        self.next_id += 1;
        send(self.writer, PluginMessage::Call { id, call })?;
        match receive(self.reader)? {
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
