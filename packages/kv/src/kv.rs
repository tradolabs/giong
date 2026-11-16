use crate::error::Error;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::{path::Path, sync::Arc};

const ENTRIES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entries");
pub type Entry = (Vec<u8>, Vec<u8>);

#[derive(Clone)]
pub struct Kv {
    database: Arc<Database>,
}

impl Kv {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let database = Database::create(path)?;
        let tx = database.begin_write()?;
        tx.open_table(ENTRIES)?;
        tx.commit()?;
        Ok(Self { database: Arc::new(database) })
    }

    pub fn namespace(&self, name: &str) -> Namespace {
        let mut prefix = (name.len() as u64).to_be_bytes().to_vec();
        prefix.extend_from_slice(name.as_bytes());
        Namespace { kv: self.clone(), prefix }
    }
}

#[derive(Clone)]
pub struct Namespace {
    kv: Kv,
    prefix: Vec<u8>,
}

impl Namespace {
    fn build_scoped_key(&self, key: &[u8]) -> Vec<u8> {
        let mut scoped = self.prefix.clone();
        scoped.extend_from_slice(key);
        scoped
    }

    pub fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        let tx = self.kv.database.begin_read()?;
        let table = tx.open_table(ENTRIES)?;
        table
            .get(self.build_scoped_key(key).as_slice())?
            .map(|value| value.value().to_vec())
            .ok_or(Error::NotFound)
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let tx = self.kv.database.begin_write()?;
        let mut table = tx.open_table(ENTRIES)?;
        table.insert(self.build_scoped_key(key).as_slice(), value)?;
        drop(table);
        tx.commit().map_err(Into::into)
    }

    pub fn delete(&self, key: &[u8]) -> Result<bool, Error> {
        let tx = self.kv.database.begin_write()?;
        let mut table = tx.open_table(ENTRIES)?;
        let removed = table.remove(self.build_scoped_key(key).as_slice())?.is_some();
        drop(table);
        tx.commit()?;
        Ok(removed)
    }

    pub fn truncate(&self) -> Result<(), Error> {
        let tx = self.kv.database.begin_write()?;
        let mut table = tx.open_table(ENTRIES)?;
        let mut keys = Vec::new();
        for entry in table.range(self.prefix.as_slice()..)? {
            let (key, _) = entry?;
            if !key.value().starts_with(&self.prefix) {
                break;
            }
            keys.push(key.value().to_vec());
        }
        for key in keys {
            table.remove(key.as_slice())?;
        }
        drop(table);
        tx.commit().map_err(Into::into)
    }

    pub fn search(&self, prefix: &[u8]) -> Result<Vec<Entry>, Error> {
        let tx = self.kv.database.begin_read()?;
        let table = tx.open_table(ENTRIES)?;
        let prefix = self.build_scoped_key(prefix);
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
