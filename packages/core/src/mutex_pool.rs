use std::{collections::HashMap, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub(crate) struct MutexPool<K> {
    locks: Mutex<HashMap<K, Arc<Mutex<()>>>>,
}

impl<K: Eq + Hash> MutexPool<K> {
    pub(crate) fn new() -> Self {
        Self { locks: Mutex::new(HashMap::new()) }
    }

    pub(crate) async fn lock(&self, key: K) -> OwnedMutexGuard<()> {
        let lock = self.locks.lock().await.entry(key).or_default().clone();
        lock.lock_owned().await
    }
}
