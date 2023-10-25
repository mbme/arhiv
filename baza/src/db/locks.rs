use std::collections::HashMap;

use anyhow::{ensure, Context, Result};

use crate::entities::{DocumentLock, DocumentLockKey, Id};
use crate::BazaEvent;

use super::kvs::{KvsEntry, KvsKey};
use super::BazaConnection;

const LOCKS_NAMESPACE: &str = "locks";

pub type Locks = HashMap<Id, DocumentLock>;

impl BazaConnection {
    pub fn list_locks(&self) -> Result<Locks> {
        let map = self
            .kvs_list(Some(LOCKS_NAMESPACE))?
            .into_iter()
            .map(|KvsEntry(key, value)| {
                (
                    Id::from(key.key),
                    serde_json::from_value::<DocumentLock>(value).expect("must parse lock value"),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(map)
    }

    pub fn get_document_lock(&self, id: &Id) -> Result<Option<DocumentLock>> {
        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        self.kvs_get(&key)
    }

    pub fn is_document_locked(&self, id: &Id) -> Result<bool> {
        let is_locked = self.get_document_lock(id)?.is_some();

        Ok(is_locked)
    }

    pub fn lock_document(&mut self, id: &Id, reason: impl Into<String>) -> Result<DocumentLock> {
        ensure!(
            !self.is_document_locked(id)?,
            "document {id} already locked"
        );

        let document = self.get_document(id)?;
        ensure!(document.is_some(), "document {id} doesn't exist");

        let reason = reason.into();

        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        let lock = DocumentLock::new(reason.clone());

        self.kvs_set(&key, &lock)?;

        self.register_event(BazaEvent::DocumentLocked {
            id: id.clone(),
            reason,
        })?;

        Ok(lock)
    }

    pub fn unlock_document(&mut self, id: &Id, key: &DocumentLockKey) -> Result<()> {
        let lock = self
            .get_document_lock(id)?
            .context("document must be locked")?;

        ensure!(lock.is_valid_key(key), "invalid lock key");

        self.unlock_document_without_key(id)?;

        Ok(())
    }

    pub fn unlock_document_without_key(&mut self, id: &Id) -> Result<()> {
        let key = KvsKey::new(LOCKS_NAMESPACE, id);

        let unlocked = self.kvs_delete(&key)?;
        ensure!(unlocked, "document {id} wasn't locked");

        self.register_event(BazaEvent::DocumentUnlocked { id: id.clone() })?;

        Ok(())
    }
}
