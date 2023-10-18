use anyhow::Result;

use crate::entities::Id;

use super::kvs::{KvsEntry, KvsKey};
use super::BazaConnection;

const SETTINGS_NAMESPACE: &str = "locks";

impl BazaConnection {
    pub fn list_locks(&self) -> Result<Vec<KvsEntry>> {
        self.kvs_list(Some(SETTINGS_NAMESPACE))
    }

    pub fn lock_document(&self, id: &Id, reason: &String) -> Result<()> {
        let key = KvsKey::new(SETTINGS_NAMESPACE, id);

        self.kvs_set(&key, reason)?;

        Ok(())
    }

    pub fn unlock_document(&self, id: &Id) -> Result<()> {
        let key = KvsKey::new(SETTINGS_NAMESPACE, id);

        self.kvs_delete(&key)?;

        Ok(())
    }
}
