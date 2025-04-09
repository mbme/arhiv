use anyhow::{bail, ensure, Context, Result};

use rs_utils::log;

use crate::entities::{DocumentLock, DocumentLockKey, Id};

use super::{BazaState, Locks};

impl BazaState {
    pub fn list_document_locks(&self) -> &Locks {
        &self.file.locks
    }

    pub fn has_document_locks(&self) -> bool {
        !self.file.locks.is_empty()
    }

    pub fn get_document_lock(&self, id: &Id) -> Option<&DocumentLock> {
        self.file.locks.get(id)
    }

    pub fn is_document_locked(&self, id: &Id) -> bool {
        self.get_document_lock(id).is_some()
    }

    pub(super) fn check_document_lock(
        &self,
        id: &Id,
        lock_key: &Option<DocumentLockKey>,
    ) -> Result<()> {
        let lock = self.get_document_lock(id);

        match (lock, lock_key) {
            (Some(lock), Some(lock_key)) => {
                if !lock.is_valid_key(lock_key) {
                    bail!("Document is locked, but an invalid lock key has been provided");
                }
            }
            (Some(_), None) => {
                bail!("Document is locked, but no lock key has been provided");
            }
            (None, Some(_)) => {
                bail!("Document isn't locked, but lock key has been provided");
            }
            (None, None) => {
                // Document isn't locked, no lock key provided
            }
        };

        Ok(())
    }

    pub fn lock_document(&mut self, id: &Id, reason: impl Into<String>) -> Result<&DocumentLock> {
        ensure!(!self.is_document_locked(id), "document {id} already locked");

        let document = self.get_document(id);
        ensure!(document.is_some(), "document {id} doesn't exist");

        let reason = reason.into();

        let lock = DocumentLock::new(reason.clone());

        self.file.locks.insert(id.clone(), lock);
        self.modified = true;
        log::trace!("State modified: locked document");

        Ok(self.get_document_lock(id).expect("lock is present"))
    }

    pub fn unlock_document(&mut self, id: &Id, key: &DocumentLockKey) -> Result<()> {
        let lock = self
            .file
            .locks
            .get(id)
            .context("Expected locked document")?;

        ensure!(lock.is_valid_key(key), "invalid lock key");

        self.unlock_document_without_key(id)?;

        Ok(())
    }

    pub fn unlock_document_without_key(&mut self, id: &Id) -> Result<()> {
        self.file
            .locks
            .remove(id)
            .context("Expected locked document")?;
        self.modified = true;
        log::trace!("State modified: unlocked document without key");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::{new_empty_document, DocumentLockKey};

    use super::BazaState;

    #[test]
    fn test_locks() {
        let mut state = BazaState::new_test_state();

        assert!(state.list_document_locks().is_empty());

        let doc1 = new_empty_document();
        assert!(!state.is_document_locked(&doc1.id));
        assert!(state.lock_document(&doc1.id, "test").is_err());
        assert!(state
            .unlock_document(&doc1.id, &DocumentLockKey::from_string("some key"))
            .is_err());
        assert!(state.unlock_document_without_key(&doc1.id).is_err());

        state.stage_document(doc1.clone(), &None).unwrap();

        state.modified = false;
        let key = state
            .lock_document(&doc1.id, "test")
            .unwrap()
            .get_key()
            .clone();
        assert!(state.is_modified());
        assert!(state.lock_document(&doc1.id, "test").is_err());

        assert!(state.has_document_locks());
        assert!(state.is_document_locked(&doc1.id));

        assert!(state
            .unlock_document(&doc1.id, &DocumentLockKey::from_string("wrong"))
            .is_err());

        state.modified = false;
        state.unlock_document(&doc1.id, &key).unwrap();
        assert!(state.is_modified());
        assert!(state.unlock_document(&doc1.id, &key).is_err());
        assert!(state.unlock_document_without_key(&doc1.id).is_err());

        assert!(!state.has_document_locks());
        assert!(!state.is_document_locked(&doc1.id));

        state.lock_document(&doc1.id, "test").unwrap();
        state.unlock_document_without_key(&doc1.id).unwrap();

        assert!(!state.has_document_locks());
        assert!(!state.is_document_locked(&doc1.id));
    }
}
