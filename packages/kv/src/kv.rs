use crate::error::Error;
use redb::{Database, ReadableDatabase, TableDefinition};
use std::{path::Path, sync::Arc};

const ENTRIES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
pub type Entry = (Vec<u8>, Vec<u8>);

/// Clone this handle rather than opening the same file more than once.
#[derive(Clone)]
pub struct Kv {
    database: Arc<Database>,
}

impl Kv {
    /// Opens an existing database or creates one. The parent directory must exist.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let database = Database::create(path)?;
        let tx = database.begin_write()?;
        tx.open_table(ENTRIES)?;
        tx.commit()?;
        Ok(Self { database: Arc::new(database) })
    }

    pub fn namespace(&self, name: &str) -> Namespace {
        // A length-prefixed name makes namespaces disjoint even when names contain separators.
        let mut prefix = (name.len() as u64).to_be_bytes().to_vec();
        prefix.extend_from_slice(name.as_bytes());
        Namespace { kv: self.clone(), prefix }
    }
}

/// A storage handle restricted to one namespace. Keys and values are opaque bytes.
#[derive(Clone)]
pub struct Namespace {
    kv: Kv,
    prefix: Vec<u8>,
}

impl Namespace {
    fn key(&self, key: &[u8]) -> Vec<u8> {
        let mut scoped = self.prefix.clone();
        scoped.extend_from_slice(key);
        scoped
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let tx = self.kv.database.begin_read()?;
        let table = tx.open_table(ENTRIES)?;
        Ok(table.get(self.key(key).as_slice())?.map(|value| value.value().to_vec()))
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let tx = self.kv.database.begin_write()?;
        {
            let mut table = tx.open_table(ENTRIES)?;
            table.insert(self.key(key).as_slice(), value)?;
        }
        tx.commit().map_err(Into::into)
    }

    pub fn delete(&self, key: &[u8]) -> Result<bool, Error> {
        let tx = self.kv.database.begin_write()?;
        let removed = {
            let mut table = tx.open_table(ENTRIES)?;
            table.remove(self.key(key).as_slice())?.is_some()
        };
        tx.commit()?;
        Ok(removed)
    }

    /// Returns entries in byte order, with namespace prefixes removed.
    pub fn list(&self, prefix: &[u8]) -> Result<Vec<Entry>, Error> {
        let tx = self.kv.database.begin_read()?;
        let table = tx.open_table(ENTRIES)?;
        let prefix = self.key(prefix);
        let mut entries = Vec::new();
        for entry in table.range(prefix.as_slice()..)? {
            let (key, value) = entry?;
            if !key.value().starts_with(&prefix) {
                break;
            }
            entries.push((key.value()[self.prefix.len()..].to_vec(), value.value().to_vec()));
        }
        Ok(entries)
    }
}
