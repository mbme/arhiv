use std::{collections::HashMap, io::Write};

use anyhow::{Context, Result, bail, ensure};

use rs_utils::{
    age::{AgeKey, AgeReader, AgeWriter},
    create_file_reader, create_file_writer, log, read_all,
};

use crate::entities::{DocumentLock, DocumentLockKey, Id};

pub type Locks = HashMap<Id, DocumentLock>;

pub struct DocumentLocksFile {
    locks: Locks,
    modified: bool,
}

impl DocumentLocksFile {
    pub fn new() -> DocumentLocksFile {
        DocumentLocksFile {
            locks: Default::default(),
            modified: false,
        }
    }

    pub fn read(file: &str, key: AgeKey) -> Result<Self> {
        log::debug!("Reading document locks from file {file}");

        let reader = create_file_reader(file)?;
        let age_reader = AgeReader::new(reader, key)?;

        let bytes = read_all(age_reader)?;
        let locks: Locks = postcard::from_bytes(&bytes).context("Failed to parse Locks")?;

        Ok(DocumentLocksFile {
            locks,
            modified: false,
        })
    }

    pub fn write(&mut self, file: &str, key: AgeKey) -> Result<()> {
        log::debug!("Writing document locks to file {file}");

        let writer = create_file_writer(file, true)?;
        let mut age_writer = AgeWriter::new(writer, key)?;

        postcard::to_io(&self.locks, &mut age_writer).context("Failed to serialize Locks")?;

        let mut writer = age_writer.finish()?;
        writer.flush()?;

        self.modified = false;

        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn list_document_locks(&self) -> &Locks {
        &self.locks
    }

    pub fn get_document_lock(&self, id: &Id) -> Option<&DocumentLock> {
        self.locks.get(id)
    }

    pub fn is_document_locked(&self, id: &Id) -> bool {
        self.get_document_lock(id).is_some()
    }

    pub fn check_document_lock(&self, id: &Id, lock_key: &Option<DocumentLockKey>) -> Result<()> {
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

    pub fn lock_document(&mut self, id: &Id, reason: String) -> Result<&DocumentLock> {
        ensure!(!self.is_document_locked(id), "document {id} already locked");

        let lock = DocumentLock::new(reason);

        self.locks.insert(id.clone(), lock);
        self.modified = true;

        Ok(self.get_document_lock(id).expect("lock is present"))
    }

    pub fn unlock_document(&mut self, id: &Id, key: &DocumentLockKey) -> Result<()> {
        let lock = self.locks.get(id).context("Expected locked document")?;

        ensure!(lock.is_valid_key(key), "invalid lock key");

        self.unlock_document_without_key(id)?;

        Ok(())
    }

    pub fn unlock_document_without_key(&mut self, id: &Id) -> Result<()> {
        self.locks.remove(id).context("Expected locked document")?;
        self.modified = true;

        Ok(())
    }
}
