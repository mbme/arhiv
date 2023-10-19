use std::collections::HashMap;

use anyhow::{ensure, Result};

use crate::entities::Id;

use super::kvs::{KvsEntry, KvsKey};
use super::BazaConnection;

const LOCKS_NAMESPACE: &str = "locks";

pub type Locks = HashMap<Id, String>;

impl BazaConnection {
    pub fn list_locks(&self) -> Result<Locks> {
        let map = self
            .kvs_list(Some(LOCKS_NAMESPACE))?
            .into_iter()
            .map(|KvsEntry(key, value)| {
                (
                    Id::from(key.key),
                    serde_json::from_value::<String>(value).expect("must parse lock value"),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(map)
    }

    pub fn lock_document(&self, id: &Id, reason: String) -> Result<()> {
        let document = self.get_document(id)?;
        ensure!(document.is_some(), "document {id} doesn't exist");

        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        self.kvs_set(&key, &reason)?;

        Ok(())
    }

    pub fn unlock_document(&self, id: &Id) -> Result<bool> {
        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        self.kvs_delete(&key)
    }
}
