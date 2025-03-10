mod blobs;
mod stats;
mod validator;

use std::{
    collections::{HashMap, HashSet},
    fs::remove_file,
    io::Read,
    time::Instant,
};

use anyhow::{ensure, Context, Result};
use thiserror::Error;

use rs_utils::{age::AgeKey, log, FsTransaction, Timestamp};

use crate::{
    baza2::{
        baza_paths::BazaPaths,
        baza_storage::{create_container_patch, STORAGE_VERSION},
        BazaInfo, BazaState, BazaStorage, DocumentHead, Filter, ListPage, Locks,
    },
    entities::{Document, DocumentKey, DocumentLock, DocumentLockKey, Id, InstanceId, Revision},
    schema::DataSchema,
};

pub use blobs::write_and_encrypt_blob;
pub use stats::{BLOBSCount, DocumentsCount};
pub use validator::ValidationError;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum StagingError {
    Validation(#[from] ValidationError),
    Other(#[from] anyhow::Error),
}

pub struct Baza {
    pub(crate) state: BazaState,
    pub(crate) state_file_modification_time: Timestamp,
    pub(crate) paths: BazaPaths,
    pub(crate) key: AgeKey,
}

impl Baza {
    pub fn create(
        instance_id: InstanceId,
        key: AgeKey,
        paths: BazaPaths,
        schema: DataSchema,
    ) -> Result<Self> {
        log::info!(
            "Creating new state file {} for instance {instance_id}",
            paths.state_file
        );

        ensure!(!paths.state_file_exists()?, "State file already exists");

        let db_files = paths.list_storage_db_files()?;
        ensure!(!db_files.is_empty(), "No existing db files found");

        // Use main db file if exists, otherwise use the first in the list
        let mut db_file = &paths.storage_main_db_file;
        if !db_files.contains(db_file) {
            db_file = &db_files[0];
        }
        log::info!("Using {db_file} db file to create new state file");

        let mut storage = BazaStorage::read_file(db_file, key.clone())?;

        let info = storage.get_info()?;

        let mut state = BazaState::new(instance_id, info.clone(), schema, HashMap::new())?;
        state.write_to_file(&paths.state_file, key.clone())?;

        let state_file_modification_time = paths.read_state_file_modification_time()?;

        log::info!("Created new state file {}", paths.state_file);

        Ok(Baza {
            state,
            state_file_modification_time,
            paths,
            key,
        })
    }

    pub fn read(key: AgeKey, paths: BazaPaths, schema: DataSchema) -> Result<Self> {
        let latest_data_version = schema.get_latest_data_version();

        let state = BazaState::read_file(&paths.state_file, key.clone(), schema)?;
        let state_file_modification_time = paths.read_state_file_modification_time()?;

        ensure!(
            state.get_info().data_version == latest_data_version,
            "State data version mismatch"
        );
        ensure!(
            state.get_info().storage_version == STORAGE_VERSION,
            "Storage version mismatch"
        );

        Ok(Baza {
            state,
            state_file_modification_time,
            paths,
            key,
        })
    }

    pub(crate) fn update_state_from_storage(&mut self) -> Result<()> {
        let mut storage =
            BazaStorage::read_file(&self.paths.storage_main_db_file, self.key.clone())?;

        let latest_snapshots_count = update_state_from_storage(&mut self.state, &mut storage)?;

        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");

            self.save_changes()?;
        }

        Ok(())
    }

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

        self.validate_staged(&document)?;

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
        let start_time = Instant::now();

        let result = self.state.list_documents(filter);

        let duration = start_time.elapsed();
        log::info!("Listed documents in {:?}", duration);

        result
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

    pub fn has_unsaved_changes(&self) -> bool {
        self.state.is_modified()
    }

    pub fn is_up_to_date_with_file(&self) -> Result<bool> {
        let is_up_to_date =
            self.state_file_modification_time == self.paths.read_state_file_modification_time()?;

        Ok(is_up_to_date)
    }

    pub fn save_changes(&mut self) -> Result<()> {
        if self.state.is_modified() {
            self.state
                .write_to_file(&self.paths.state_file, self.key.clone())?;
            self.state_file_modification_time = self.paths.read_state_file_modification_time()?;
            log::info!("Saved state changes to the file");
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<bool> {
        self.save_changes()?;

        if !self.has_staged_documents() {
            log::debug!("Can't commit: nothing to commit");
            return Ok(false);
        }

        if self.state.has_document_locks() {
            log::debug!("Can't commit: some documents are locked");
            return Ok(false);
        }

        let mut fs_tx = FsTransaction::new();

        // backup db file
        let old_db_file = fs_tx.move_to_backup(self.paths.storage_main_db_file.clone())?;

        // open old db file
        let storage = BazaStorage::read_file(&old_db_file, self.key.clone())?;

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

            fs_tx.move_file(state_blob_path, storage_blob_path, true)?;
        }

        // write changes to db file
        let mut patch = create_container_patch(new_snapshots.into_iter())?;
        for key in self.get_storage_keys_to_erase(&storage)? {
            patch.insert(key, None);
        }
        storage.patch_and_save_to_file(&self.paths.storage_main_db_file, patch)?;

        // backup state file
        fs_tx.move_to_backup(self.paths.state_file.clone())?;

        // write changes to state file
        self.state
            .write_to_file(&self.paths.state_file, self.key.clone())?;

        fs_tx.commit()?;
        log::info!("Commit: finished");

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

    /// collect keys of storage documents that are known to be erased in the state
    fn get_storage_keys_to_erase<R: Read>(&self, storage: &BazaStorage<R>) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        for key in storage.index.iter() {
            if let Some(head) = self.state.get_document(&key.id) {
                if head.is_original_erased() && key.rev.is_older_than(head.get_revision()) {
                    keys.push(key.serialize());
                }
            }
        }

        Ok(keys)
    }
}

impl Drop for Baza {
    fn drop(&mut self) {
        if self.has_unsaved_changes() {
            log::error!("Dropping Baza with unsaved changes");
        }
    }
}

fn update_state_from_storage<R: Read>(
    state: &mut BazaState,
    storage: &mut BazaStorage<R>,
) -> Result<usize> {
    if state.has_staged_documents() {
        return Ok(0);
    }

    let storage_info = storage.get_info()?;
    ensure!(
        storage_info == state.get_info(),
        "state info and storage info must match"
    );

    let mut latest_snapshot_keys: HashSet<DocumentKey> = HashSet::new();

    // compare storage index with state
    for (id, index_revs) in storage.index.as_index_map() {
        let index_rev = index_revs
            .iter()
            .next()
            .context("index revs must not be empty")?;

        let document_head = if let Some(document_head) = state.get_document(id) {
            document_head
        } else {
            add_keys(&mut latest_snapshot_keys, id, index_revs.iter());
            continue;
        };

        ensure!(
            document_head.is_committed(),
            "Document {id} must be committed"
        );

        let state_rev = document_head.get_revision();

        if state_rev > index_rev {
            continue;
        }

        if state_rev < index_rev {
            add_keys(&mut latest_snapshot_keys, id, index_revs.iter());
            continue;
        }

        let all_state_revs = document_head.get_original_revisions().collect();
        // conflicting revs
        add_keys(
            &mut latest_snapshot_keys,
            id,
            index_revs.difference(&all_state_revs),
        );
    }

    let latest_snapshots_count = latest_snapshot_keys.len();

    // read documents from storage & update state if needed
    while !latest_snapshot_keys.is_empty() {
        let (ref key, ref raw_document) = storage.next().context("No records in the storage")??;

        if !latest_snapshot_keys.contains(key) {
            continue;
        }

        let document: Document =
            serde_json::from_str(raw_document).context("Failed to parse raw document")?;

        state.insert_snapshot(document)?;

        latest_snapshot_keys.remove(key);
    }

    Ok(latest_snapshots_count)
}

fn add_keys<'r>(
    keys: &mut HashSet<DocumentKey>,
    id: &Id,
    revs: impl Iterator<Item = &'r &'r Revision>,
) {
    keys.extend(revs.map(|rev| DocumentKey::new(id.clone(), (*rev).clone())));
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use rs_utils::age::AgeKey;

    use crate::{
        baza2::{baza_storage::create_test_storage, BazaState, DocumentHead},
        tests::new_document,
    };

    use super::update_state_from_storage;

    #[test]
    fn test_update_state_from_storage() {
        let key = AgeKey::generate_age_x25519_key();

        let doc_a = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a1 = doc_a.clone().with_rev(json!({ "b": 1 }));

        let doc_b = new_document(json!({})).with_rev(json!({ "b": 1 }));
        let doc_b1 = doc_b.clone().with_rev(json!({ "b": 2 }));

        let doc_c = new_document(json!({})).with_rev(json!({ "c": 3 }));

        let mut state = BazaState::new_test_state();
        state.insert_snapshots(vec![doc_a.clone(), doc_b.clone()]);

        let mut storage = create_test_storage(
            key.clone(),
            &vec![
                doc_a.clone(),
                doc_a1.clone(),
                doc_b.clone(),
                doc_b1.clone(),
                doc_c.clone(),
            ],
        );

        let changes = update_state_from_storage(&mut state, &mut storage).unwrap();
        assert_eq!(changes, 3);

        assert_eq!(
            *state.get_document(&doc_a.id).unwrap(),
            DocumentHead::new_conflict([doc_a.clone(), doc_a1.clone(),].into_iter()).unwrap(),
        );

        assert_eq!(
            *state.get_document(&doc_b.id).unwrap(),
            DocumentHead::new(doc_b1),
        );

        assert_eq!(
            *state.get_document(&doc_c.id).unwrap(),
            DocumentHead::new(doc_c),
        );
    }
}
