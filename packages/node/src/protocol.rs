use crate::host::Flow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const VERSION: u16 = 2;
pub const MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostMessage {
    Start { version: u16, operation: Operation },
    Reply { id: u64, result: Result<Value, String> },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Operation {
    Metadata,
    Execute { properties: Value },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginMessage {
    Ready { version: u16 },
    Call { id: u64, call: Call },
    Log { level: LogLevel, message: String },
    Finish { result: Result<Value, String> },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Info,
    Error,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Call {
    GetSecret { vault: Option<String>, key: String },
    EmitChanged,
    SynchronizeFlow { definitions: Vec<Flow> },
    GetScope,
    GetOutput { name: String },
    GetStorage { key: String },
    PutStorage { key: String, value: Vec<u8> },
    DeleteStorage { key: String },
    SearchStorage { prefix: String },
    GetSourceWorkspace { name: String },
}

pub fn encode_frame(message: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(message).map_err(|_| "Could not encode plugin message")?;
    if bytes.len() >= MAX_FRAME_BYTES {
        return Err("Plugin message is too large".into());
    }
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn decode_frame<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    if bytes.is_empty() {
        return Err("Plugin connection closed before a complete message".into());
    }
    if bytes.len() > MAX_FRAME_BYTES {
        return Err("Plugin message is too large".into());
    }
    if bytes.last() != Some(&b'\n') {
        return Err("Incomplete plugin message".into());
    }
    serde_json::from_slice(bytes).map_err(|_| "Invalid plugin message".into())
}
