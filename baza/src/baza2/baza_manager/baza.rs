use std::{collections::HashSet, fs::remove_file, io::Read};

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

use rs_utils::{age::AgeKey, log, FsTransaction, LockFile, Timestamp};

use crate::{
    baza2::{
        baza_storage::{create_container_patch, merge_storages_to_file, BazaFileStorage},
        BazaInfo, BazaState, BazaStorage, DocumentHead, Filter, ListPage, Locks,
    },
    entities::{
        BLOBId, Document, DocumentKey, DocumentLock, DocumentLockKey, Id, InstanceId, Revision,
    },
    schema::DataSchema,
    validator::{ValidationError, Validator},
};

use super::{baza_paths::BazaPaths, update_state_from_storage};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum StagingError {
    Validation(#[from] ValidationError),
    Other(#[from] anyhow::Error),
}

pub struct Baza {
    pub(super) _lock: LockFile,
    pub(crate) state: BazaState,
    pub(crate) paths: BazaPaths,
    pub(crate) key: AgeKey,
}

impl Baza {
    #[cfg(test)]
    pub fn create_storage_file(&self, file_path: &str, docs: &[Document]) {
        use rs_utils::create_file_writer;

        use crate::baza2::baza_storage::create_storage;

        let mut storage_writer = create_file_writer(file_path, false).unwrap();
        create_storage(&mut storage_writer, self.key.clone(), self.get_info(), docs).unwrap();
    }

    pub fn get_info(&self) -> &BazaInfo {
        self.state.get_info()
    }

    pub fn get_instance_id(&self) -> &InstanceId {
        self.state.get_instance_id()
    }

    pub fn get_data_version(&self) -> u8 {
        self.state.get_info().data_version
    }

    pub fn get_single_latest_revision(&self) -> &Revision {
        self.state.get_single_latest_revision()
    }

    pub fn get_schema(&self) -> &DataSchema {
        self.state.get_schema()
    }

    pub fn get_storage_dir(&self) -> &str {
        &self.paths.storage_dir
    }

    pub fn find_last_modification_time(&self) -> Option<Timestamp> {
        self.state.find_last_modification_time()
    }

    pub fn list_document_locks(&self) -> &Locks {
        self.state.list_document_locks()
    }

    pub fn has_document_locks(&self) -> bool {
        self.state.has_document_locks()
    }

    pub fn is_document_locked(&self, id: &Id) -> bool {
        self.state.is_document_locked(id)
    }

    pub fn lock_document(&mut self, id: &Id, reason: impl Into<String>) -> Result<&DocumentLock> {
        let reason = reason.into();
        log::debug!("Locking document {id}: {reason}");
        self.state.lock_document(id, reason)
    }

    pub fn unlock_document(&mut self, id: &Id, key: &DocumentLockKey) -> Result<()> {
        log::debug!("Unlocking document {id}");

        self.state.unlock_document(id, key)
    }

    pub fn unlock_document_without_key(&mut self, id: &Id) -> Result<()> {
        log::info!("Unlocking document {id} without a key");

        self.state.unlock_document_without_key(id)
    }

    pub fn get_document(&self, id: &Id) -> Option<&DocumentHead> {
        self.state.get_document(id)
    }

    pub fn must_get_document(&self, id: &Id) -> Result<&Document> {
        self.state.must_get_document(id)
    }

    pub fn stage_document(
        &mut self,
        document: Document,
        lock_key: &Option<DocumentLockKey>,
    ) -> std::result::Result<&Document, StagingError> {
        log::debug!("Staging document {}", &document.id);

        Validator::new(self as &Baza).validate_staged(&document)?;

        let document = self.state.stage_document(document, lock_key)?;

        Ok(document)
    }

    pub fn erase_document(&mut self, id: &Id) -> Result<()> {
        log::debug!("Erasing document {id}");

        self.state.erase_document(id)
    }

    pub fn has_staged_documents(&self) -> bool {
        self.state.has_staged_documents()
    }

    pub fn iter_documents(&self) -> impl Iterator<Item = &DocumentHead> {
        self.state.iter_documents()
    }

    #[cfg(test)]
    pub fn insert_snapshot(&mut self, document: Document) -> Result<()> {
        self.state.insert_snapshot(document)
    }

    pub fn list_documents(&self, filter: &Filter) -> Result<ListPage> {
        self.state.list_documents(filter)
    }

    pub fn find_document_backrefs(&self, id: &Id) -> HashSet<Id> {
        self.state.find_document_backrefs(id)
    }

    pub fn find_document_collections(&self, id: &Id) -> HashSet<Id> {
        self.state.find_document_collections(id)
    }

    pub fn update_document_collections(
        &mut self,
        document_id: &Id,
        collections: &Vec<Id>,
    ) -> Result<()> {
        log::debug!("Updating collections of document {document_id}");

        self.state
            .update_document_collections(document_id, collections)
    }

    pub(crate) fn open_storage<'s>(&self, file_path: &str) -> Result<BazaFileStorage<'s>> {
        BazaStorage::read_file(file_path, self.key.clone())
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.state.is_modified()
    }

    pub fn save_changes(&mut self) -> Result<()> {
        if self.state.is_modified() {
            self.state
                .write_to_file(&self.paths.state_file, self.key.clone())?;
            log::info!("Saved state changes to the file");
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<bool> {
        self.save_changes()?;

        self.merge_storages()?;

        if !self.has_staged_documents() {
            log::debug!("Can't commit: nothing to commit");
            return Ok(false);
        }

        if self.state.has_document_locks() {
            log::debug!("Can't commit: some documents are locked");
            return Ok(false);
        }

        let mut tx = FsTransaction::new();

        // backup db file
        let old_db_file = tx.move_to_backup(self.paths.storage_main_db_file.clone())?;

        // open old db file
        let storage = self.open_storage(&old_db_file)?;

        // update state
        self.state.commit()?;

        // collect snapshots that aren't present in the storage
        let new_snapshots = self
            .state
            .iter_documents()
            .flat_map(|head| head.iter_original_snapshots())
            .filter(|document| !storage.contains(&DocumentKey::for_document(document)))
            .collect::<Vec<_>>();
        log::info!("Commit: {} new document snapshots", new_snapshots.len());

        // collect new blobs that are used by new snapshots
        let new_blobs = self.collect_new_blobs(&new_snapshots)?;
        log::info!("Commit: {} new BLOBs", new_blobs.len());

        // move blobs
        for new_blob_id in new_blobs {
            let state_blob_path = self.paths.get_state_blob_path(&new_blob_id);
            let storage_blob_path = self.paths.get_storage_blob_path(&new_blob_id);

            tx.move_file(state_blob_path, storage_blob_path, true)?;
        }

        // write changes to db file
        let mut patch = create_container_patch(new_snapshots.into_iter())?;
        for key in get_storage_keys_to_erase(&storage, &mut self.state)? {
            patch.insert(key, None);
        }
        storage.patch_and_save_to_file(&self.paths.storage_main_db_file, patch)?;

        // backup state file
        tx.move_to_backup(self.paths.state_file.clone())?;

        // write changes to state file
        self.state
            .write_to_file(&self.paths.state_file, self.key.clone())?;

        tx.commit()?;
        log::info!("Commit: finished");

        self.update_state_from_storage()?;
        self.remove_unused_storage_blobs()?;

        // remove unused state BLOBs if any
        let unused_state_blobs = self.paths.list_state_blobs()?;
        if !unused_state_blobs.is_empty() {
            log::info!("Removing {} unused state BLOBs", unused_state_blobs.len());

            for blob_id in unused_state_blobs {
                let file_path = self.paths.get_state_blob_path(&blob_id);
                remove_file(file_path).context("Failed to remove unused state BLOB")?;
            }
        }

        Ok(true)
    }

    fn collect_new_blobs(&self, new_snapshots: &[&Document]) -> Result<HashSet<BLOBId>> {
        let mut new_blobs = HashSet::new();

        for document in new_snapshots {
            let key = document.create_key();
            let refs = self
                .state
                .get_document_snapshot_refs(&key)
                .context(anyhow!("Can't find document refs for {key:?}"))?;

            for blob_id in &refs.blobs {
                if new_blobs.contains(blob_id) {
                    continue;
                }

                if self.paths.storage_blob_exists(blob_id)? {
                    continue;
                }

                new_blobs.insert(blob_id.clone());
            }
        }

        Ok(new_blobs)
    }

    pub(super) fn update_state_from_storage(&mut self) -> Result<()> {
        let mut storage = self.open_storage(&self.paths.storage_main_db_file)?;

        let latest_snapshots_count = update_state_from_storage(&mut self.state, &mut storage)?;

        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");
            self.state
                .write_to_file(&self.paths.state_file, self.key.clone())?;
        }

        Ok(())
    }

    pub(super) fn merge_storages(&self) -> Result<()> {
        let db_files = self.paths.list_storage_db_files()?;

        if db_files.is_empty() {
            log::debug!("No existing db files found");
            return Ok(());
        }

        let main_db_file = &self.paths.storage_main_db_file;
        if db_files.len() == 1 && db_files[0] == *main_db_file {
            log::debug!("There's only main db file");
            return Ok(());
        }

        // if more than 1 storage
        // or if no main storage file

        log::info!("Merging {} db files into one", db_files.len());

        let mut tx = FsTransaction::new();

        // backup db files and open storages
        let storages = db_files
            .iter()
            .map(|db_file| {
                let new_db_file = tx.move_to_backup(db_file)?;

                self.open_storage(&new_db_file)
                    .context(anyhow!("Failed to open storage for db {db_file}"))
            })
            .collect::<Result<Vec<_>>>()?;

        merge_storages_to_file(self.get_info(), storages, main_db_file)?;

        tx.commit()?;

        Ok(())
    }

    pub(super) fn remove_unused_storage_blobs(&self) -> Result<()> {
        let blob_refs = self.state.get_all_blob_refs();
        let storage_blobs = self.paths.list_storage_blobs()?;

        // warn about missing storage BLOBs if any
        let missing_blobs = blob_refs.difference(&storage_blobs).collect::<Vec<_>>();
        if !missing_blobs.is_empty() {
            log::warn!("There are {} missing BLOBs", missing_blobs.len());
            log::trace!("Missing BLOBs: {missing_blobs:?}");
        }

        // remove unused storage BLOBs if any
        let unused_storage_blobs = storage_blobs.difference(&blob_refs).collect::<Vec<_>>();
        if !unused_storage_blobs.is_empty() {
            log::info!(
                "Removing {} unused storage BLOBs",
                unused_storage_blobs.len()
            );

            for blob_id in unused_storage_blobs {
                let file_path = self.paths.get_storage_blob_path(blob_id);
                remove_file(file_path).context("Failed to remove unused storage BLOB")?;
            }
        }

        Ok(())
    }
}

impl Drop for Baza {
    fn drop(&mut self) {
        if self.has_unsaved_changes() {
            log::error!("Dropping Baza with unsaved changes");
        }
    }
}

/// collect keys of storage documents that are known to be erased in the state
fn get_storage_keys_to_erase<R: Read>(
    storage: &BazaStorage<R>,
    state: &mut BazaState,
) -> Result<Vec<String>> {
    let mut keys = Vec::new();
    for key in storage.index.iter() {
        if let Some(head) = state.get_document(&key.id) {
            if head.is_original_erased() && key.rev.is_older_than(head.get_revision()) {
                keys.push(key.serialize());
            }
        }
    }

    Ok(keys)
}
