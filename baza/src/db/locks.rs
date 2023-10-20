use std::collections::HashMap;

use anyhow::{ensure, Result};

use crate::entities::Id;
use crate::BazaEvent;

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

    pub fn is_document_locked(&self, id: &Id) -> Result<bool> {
        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        let is_locked = self.kvs_get::<String>(&key)?.is_some();

        Ok(is_locked)
    }

    pub fn lock_document(&mut self, id: &Id, reason: String) -> Result<()> {
        ensure!(
            !self.is_document_locked(id)?,
            "document {id} already locked"
        );

        let document = self.get_document(id)?;
        ensure!(document.is_some(), "document {id} doesn't exist");

        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        self.kvs_set(&key, &reason)?;

        self.register_event(BazaEvent::DocumentLocked {
            id: id.clone(),
            reason,
        })?;

        Ok(())
    }

    pub fn unlock_document(&mut self, id: &Id) -> Result<bool> {
        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        let unlocked = self.kvs_delete(&key)?;

        if unlocked {
            self.register_event(BazaEvent::DocumentUnlocked { id: id.clone() })?;
        }

        Ok(unlocked)
    }
}
