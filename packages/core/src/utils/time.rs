use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
