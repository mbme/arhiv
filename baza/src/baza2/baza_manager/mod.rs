mod baza_paths;
mod blobs;
pub mod stats;

use std::{
    collections::{HashMap, HashSet},
    fs::remove_file,
    io::Read,
};

use anyhow::{anyhow, ensure, Context, Result};
use thiserror::Error;

use rs_utils::{
    age::{encrypt_and_write_file, read_and_decrypt_file, AgeKey},
    get_file_modification_time, log, FsTransaction, LockFile, SecretString, Timestamp,
};

use crate::{
    baza2::baza_storage::create_container_patch,
    entities::{
        BLOBId, Document, DocumentKey, DocumentLock, DocumentLockKey, Id, InstanceId, Revision,
    },
    schema::DataSchema,
    validator::{ValidationError, Validator},
    DocumentExpert,
};

use baza_paths::BazaPaths;

use super::{
    baza_state::Locks,
    baza_storage::{
        create_empty_storage_file, merge_storages_to_file, BazaFileStorage, STORAGE_VERSION,
    },
    BazaInfo, BazaState, BazaStorage, DocumentHead, Filter, ListPage,
};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum StagingError {
    Validation(#[from] ValidationError),
    Other(#[from] anyhow::Error),
}

pub struct BazaManager {
    schema: DataSchema,
    paths: BazaPaths,
    key: Option<AgeKey>,
}

impl BazaManager {
    pub const MIN_LOGIN_LENGTH: usize = 2;
    pub const MIN_PASSWORD_LENGTH: usize = AgeKey::MIN_PASSWORD_LEN;

    pub fn new(storage_dir: String, state_dir: String, schema: DataSchema) -> Self {
        let paths = BazaPaths::new(storage_dir, state_dir);

        BazaManager {
            schema,
            paths,
            key: None,
        }
    }

    pub fn open(&self) -> Result<Baza> {
        log::info!("Opening baza {}", self.paths);

        ensure!(self.storage_exists()?, "Storage doesn't exist");

        let key = self.key.as_ref().context("Key is missing")?;

        let lock = LockFile::wait_for_lock(&self.paths.lock_file)?;

        let state = if self.paths.state_file_exists()? {
            log::info!("Reading state file {}", self.paths.state_file);

            let state =
                BazaState::read_file(&self.paths.state_file, key.clone(), self.schema.clone())?;

            ensure!(
                state.get_info().data_version == self.schema.get_latest_data_version(),
                "State data version mismatch"
            );
            ensure!(
                state.get_info().storage_version == STORAGE_VERSION,
                "Storage version mismatch"
            );

            state
        } else {
            log::info!("Creating new state file {}", self.paths.state_file);

            self.create_state()?
        };

        let mut baza = Baza {
            _lock: lock,
            state,
            key: key.clone(),
            paths: self.paths.clone(),
        };

        baza.merge_storages()?;

        if !baza.has_staged_documents() {
            baza.update_state_from_storage()?;
            baza.remove_unused_storage_blobs()?;
        }

        Ok(baza)
    }

    pub fn storage_exists(&self) -> Result<bool> {
        let have_storage_files = !self.paths.list_storage_db_files()?.is_empty();
        let have_key_file = self.paths.key_file_exists()?;

        Ok(have_storage_files && have_key_file)
    }

    pub fn get_state_file_modification_time(&self) -> Result<Timestamp> {
        get_file_modification_time(&self.paths.state_file)
    }

    pub fn create(&mut self, login: String, password: SecretString) -> Result<()> {
        log::info!("Creating {login} baza in {}", self.paths);

        ensure!(
            login.len() >= Self::MIN_LOGIN_LENGTH,
            "Login should be at least {} characters long",
            Self::MIN_LOGIN_LENGTH
        );

        self.paths.ensure_dirs_exist()?;

        ensure!(!self.paths.key_file_exists()?, "Key file already exists");
        ensure!(
            !self.paths.state_file_exists()?,
            "State file already exists"
        );

        let _lock = LockFile::wait_for_lock(&self.paths.lock_file)?;

        let key_file_key = AgeKey::from_password(password)?;
        let key = self.generate_key_file(key_file_key)?;

        let info = BazaInfo {
            login: login.clone(),
            data_version: self.schema.get_latest_data_version(),
            storage_version: STORAGE_VERSION,
        };
        create_empty_storage_file(&self.paths.storage_main_db_file, key.clone(), &info)?;

        self.key = Some(key);

        log::info!(
            "Created new {login} main storage file {}",
            self.paths.storage_main_db_file
        );

        Ok(())
    }

    fn generate_key_file(&self, key_file_key: AgeKey) -> Result<AgeKey> {
        let key_file = AgeKey::generate_age_x25519_key();

        encrypt_and_write_file(
            &self.paths.key_file,
            key_file_key,
            key_file.serialize().into(),
        )?;

        log::debug!("Generated new key file {}", self.paths.key_file);

        Ok(key_file)
    }

    pub fn read_key_file(&mut self, password: SecretString) -> Result<()> {
        log::debug!("Reading key file {}", self.paths.key_file);

        let key_file_key = AgeKey::from_password(password)?;

        let key = read_and_decrypt_file(&self.paths.key_file, key_file_key)?;

        let key = AgeKey::from_age_x25519_key(key.try_into()?)?;

        self.key = Some(key);

        Ok(())
    }

    pub fn change_key_file_password(
        &self,
        old_password: SecretString,
        new_password: SecretString,
    ) -> Result<()> {
        log::warn!("Changing key file password {}", self.paths.key_file);

        let old_key_file_key = AgeKey::from_password(old_password)?;
        let data = read_and_decrypt_file(&self.paths.key_file, old_key_file_key)?;

        let new_key_file_key = AgeKey::from_password(new_password)?;

        let mut fs_tx = FsTransaction::new();
        fs_tx.move_to_backup(&self.paths.key_file)?;
        encrypt_and_write_file(&self.paths.key_file, new_key_file_key, data)?;
        fs_tx.commit()?;

        Ok(())
    }

    fn create_state(&self) -> Result<BazaState> {
        log::info!("Creating new state file {}", self.paths.state_file);

        let db_files = self.paths.list_storage_db_files()?;
        ensure!(!db_files.is_empty(), "No existing db files found");

        // Use main db file if exists, otherwise use the first in the list
        let mut db_file = &self.paths.storage_main_db_file;
        if !db_files.contains(db_file) {
            db_file = &db_files[0];
        }
        log::info!("Using {db_file} db file to create new state file");

        let key = self.key.as_ref().context("Key is missing")?;
        let mut storage = BazaStorage::read_file(db_file, key.clone())?;

        let info = storage.get_info()?;

        let mut state = BazaState::new(
            InstanceId::generate(),
            info.clone(),
            self.schema.clone(),
            HashMap::new(),
        )?;
        state.write_to_file(&self.paths.state_file, key.clone())?;

        log::info!("Created new state file {}", self.paths.state_file);

        Ok(state)
    }

    #[cfg(test)]
    pub fn new_for_tests(test_dir: &str) -> Self {
        let schema = DataSchema::new_test_schema();

        let mut manager = BazaManager::new(
            format!("{test_dir}/storage"),
            format!("{test_dir}/state"),
            schema,
        );

        manager
            .create("test login".to_string(), "test password".into())
            .expect("must create test baza");

        manager
    }

    pub fn get_schema(&self) -> &DataSchema {
        &self.schema
    }

    pub fn get_document_expert(&self) -> DocumentExpert {
        DocumentExpert::new(&self.schema)
    }
}

pub struct Baza {
    _lock: LockFile,
    state: BazaState,
    paths: BazaPaths,
    key: AgeKey,
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

    fn open_storage<'s>(&self, file_path: &str) -> Result<BazaFileStorage<'s>> {
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

    fn update_state_from_storage(&mut self) -> Result<()> {
        let mut storage = self.open_storage(&self.paths.storage_main_db_file)?;

        let latest_snapshots_count = update_state_from_storage(&mut self.state, &mut storage)?;

        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");
            self.state
                .write_to_file(&self.paths.state_file, self.key.clone())?;
        }

        Ok(())
    }

    fn merge_storages(&self) -> Result<()> {
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

    fn remove_unused_storage_blobs(&self) -> Result<()> {
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

fn add_keys<'r>(
    keys: &mut HashSet<DocumentKey>,
    id: &Id,
    revs: impl Iterator<Item = &'r &'r Revision>,
) {
    keys.extend(revs.map(|rev| DocumentKey::new(id.clone(), (*rev).clone())));
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use rs_utils::{age::AgeKey, dir_exists, file_exists, TempFile};

    use crate::{
        baza2::{baza_manager::BazaManager, baza_storage::create_test_storage, DocumentHead},
        tests::{new_document, new_empty_document},
    };

    use super::{update_state_from_storage, BazaState};

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

    #[test]
    fn test_commit() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open().unwrap();

        assert!(dir_exists(&baza.paths.storage_dir).unwrap());
        assert!(dir_exists(&baza.paths.state_dir).unwrap());

        assert!(file_exists(&baza.paths.state_file).unwrap());
        assert!(file_exists(&baza.paths.storage_main_db_file).unwrap());

        let blob1_file = temp_dir.new_child("blob1");
        blob1_file.write_str("blob1").unwrap();

        let blob2_file = temp_dir.new_child("blob2");
        blob2_file.write_str("blob2").unwrap();

        let blob1 = baza.add_blob(&blob1_file.path).unwrap();
        let blob2 = baza.add_blob(&blob2_file.path).unwrap();

        assert!(
            baza.stage_document(new_document(json!({ "blob": "unknown" })), &None)
                .is_err(),
            "Can't stage document that references unknown BLOB"
        );
        assert!(
            baza.stage_document(new_document(json!({ "ref": "unknown" })), &None)
                .is_err(),
            "Can't stage document that references unknown document"
        );

        baza.stage_document(new_document(json!({ "blob": blob1 })), &None)
            .unwrap();

        assert!(baza.has_staged_documents());

        baza.insert_snapshot(new_document(json!({})).with_rev(json!({ "a": 1 })))
            .unwrap();

        baza.commit().unwrap();
        drop(baza);

        let baza = manager.open().unwrap();

        // ensure new BLOB is committed
        assert!(baza.paths.storage_blob_exists(&blob1).unwrap());

        // ensure unused state BLOB is removed
        assert!(!baza.paths.storage_blob_exists(&blob2).unwrap());
        assert!(baza.paths.list_state_blobs().unwrap().is_empty());

        assert!(!baza.has_staged_documents());

        let storage = baza.open_storage(&baza.paths.storage_main_db_file).unwrap();
        assert_eq!(storage.index.len(), 2);
    }

    #[test]
    fn test_removes_erased_snapshots_from_storage() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open().unwrap();

        // Create and stage a new document with a BLOB
        let blob_file = temp_dir.new_child("blob");
        blob_file.write_str("blob_content").unwrap();
        let blob_id = baza.add_blob(&blob_file.path).unwrap();

        let doc_a1 = new_document(json!({ "blob": blob_id }));
        baza.stage_document(doc_a1.clone(), &None).unwrap();
        baza.commit().unwrap();
        drop(baza);

        // Ensure the document and BLOB are in storage
        let mut baza = manager.open().unwrap();
        let doc_a1_key = baza.get_document(&doc_a1.id).unwrap().create_key();
        let storage = baza.open_storage(&baza.paths.storage_main_db_file).unwrap();
        assert!(storage.contains(&doc_a1_key));
        assert!(baza.paths.storage_blob_exists(&blob_id).unwrap());

        // Erase the document and commit
        let mut doc_a2 = doc_a1.clone();
        doc_a2.erase();
        baza.state.stage_document(doc_a2.clone(), &None).unwrap();
        baza.commit().unwrap();
        drop(baza);

        // Reopen storage and check the snapshot and BLOB are removed
        let baza = manager.open().unwrap();
        let doc_a2_key = baza.get_document(&doc_a2.id).unwrap().create_key();
        let storage = baza.open_storage(&baza.paths.storage_main_db_file).unwrap();
        assert!(!storage.contains(&doc_a1_key));
        assert!(storage.contains(&doc_a2_key));
        assert!(!baza.paths.storage_blob_exists(&blob_id).unwrap());
    }

    #[test]
    fn test_merge_storages() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open().unwrap();

        let db_file_1 = baza.paths.get_storage_file("db1");
        let db_file_2 = baza.paths.get_storage_file("db2");

        let doc_a1 = new_document(json!({ "test": "a" })).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "a": 2 }));

        let doc_b1 = new_document(json!({ "test": "b" })).with_rev(json!({ "a": 1 }));

        baza.create_storage_file(&db_file_1, &[doc_a1.clone(), doc_b1.clone()]);
        baza.create_storage_file(&db_file_2, &[doc_a1.clone(), doc_a2.clone()]);

        assert!(file_exists(&db_file_1).unwrap());
        assert!(file_exists(&db_file_2).unwrap());

        baza.stage_document(new_document(json!({})), &None).unwrap();

        baza.commit().unwrap();
        drop(baza);

        let baza = manager.open().unwrap();

        assert!(!file_exists(&db_file_1).unwrap());
        assert!(!file_exists(&db_file_2).unwrap());

        assert_eq!(baza.iter_documents().count(), 3);

        let storage = baza.open_storage(&baza.paths.storage_main_db_file).unwrap();
        assert_eq!(storage.index.len(), 4);
    }

    #[test]
    fn test_preserve_state() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);

        {
            let mut baza = manager.open().unwrap();
            let document = new_empty_document();
            baza.stage_document(document.clone(), &None).unwrap();
            baza.lock_document(&document.id, "test").unwrap();
            baza.save_changes().unwrap();
        }

        {
            let baza = manager.open().unwrap();

            assert!(baza.has_staged_documents());
            assert!(baza.has_document_locks());
        }
    }

    #[test]
    fn test_open_storage() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let storage_dir = manager.paths.storage_dir.clone();
        let state_dir = manager.paths.state_dir.clone();
        let schema = manager.schema.clone();

        {
            let mut baza = manager.open().unwrap();
            let document = new_empty_document();
            baza.stage_document(document.clone(), &None).unwrap();
            baza.lock_document(&document.id, "test").unwrap();
            baza.save_changes().unwrap();
        }

        {
            let mut manager = BazaManager::new(storage_dir, state_dir, schema);

            assert!(manager.open().is_err(), "Can't open without password");
            assert!(
                manager.read_key_file("wrong password".into()).is_err(),
                "Can't open with wrong password"
            );
            manager.read_key_file("test password".into()).unwrap();

            let baza = manager.open().unwrap();

            assert!(baza.has_staged_documents());
            assert!(baza.has_document_locks());
        }
    }

    #[test]
    fn test_change_password() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let storage_dir = manager.paths.storage_dir.clone();
        let state_dir = manager.paths.state_dir.clone();
        let schema = manager.schema.clone();

        manager
            .change_key_file_password("test password".into(), "new password".into())
            .unwrap();

        {
            let mut manager = BazaManager::new(storage_dir, state_dir, schema);

            assert!(
                manager.read_key_file("test password".into()).is_err(),
                "Can't open with old password"
            );
            manager.read_key_file("new password".into()).unwrap();
        }
    }
}
