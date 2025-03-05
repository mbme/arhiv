mod baza;
mod blobs;
pub mod stats;

use std::{
    collections::{HashMap, HashSet},
    fs::remove_file,
    io::Read,
    sync::RwLock,
};

use anyhow::{anyhow, ensure, Context, Result};

use rs_utils::{
    age::{encrypt_and_write_file, read_and_decrypt_file, AgeKey},
    file_exists, get_file_modification_time, log, FsTransaction, LockFile, SecretString, Timestamp,
};

use crate::{
    baza2::baza_storage::{create_container_patch, merge_storages_to_file},
    entities::{BLOBId, Document, DocumentKey, Id, InstanceId, Revision},
    schema::DataSchema,
    DocumentExpert,
};

pub use baza::{Baza, StagingError};
use blobs::write_and_encrypt_blob;

use super::{
    baza_paths::BazaPaths,
    baza_storage::{create_empty_storage_file, BazaFileStorage, STORAGE_VERSION},
    BazaInfo, BazaState, BazaStorage,
};

pub struct BazaManager {
    schema: DataSchema,
    pub(crate) paths: BazaPaths,
    pub(crate) key: RwLock<Option<AgeKey>>,
}

impl BazaManager {
    pub const MIN_PASSWORD_LENGTH: usize = AgeKey::MIN_PASSWORD_LEN;

    pub fn new(storage_dir: String, state_dir: String, schema: DataSchema) -> Self {
        let paths = BazaPaths::new(storage_dir, state_dir);

        BazaManager {
            schema,
            paths,
            key: RwLock::new(None),
        }
    }

    pub fn open(&self) -> Result<Baza> {
        log::info!("Opening baza {}", self.paths);

        ensure!(self.storage_exists()?, "Storage doesn't exist");

        let key = self
            .key
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the key: {err}"))?;
        let key = key.as_ref().context("Key is missing")?;

        let lock = LockFile::wait_for_lock(&self.paths.lock_file)?;

        let mut state = if self.paths.state_file_exists()? {
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

            self.create_state(InstanceId::generate())?
        };

        self.merge_storages()?;

        if !state.has_staged_documents() {
            self.update_state_from_storage(&mut state)?;
            self.remove_unused_storage_blobs(&state)?;
        }

        let baza = Baza {
            _lock: lock,
            state,
            key: key.clone(),
            paths: self.paths.clone(),
        };

        Ok(baza)
    }

    pub fn storage_exists(&self) -> Result<bool> {
        let storage_dir_exists = self.paths.storage_dir_exists()?;
        if !storage_dir_exists {
            return Ok(false);
        }

        let have_storage_db_files = !self.paths.list_storage_db_files()?.is_empty();
        let have_key_file = self.paths.key_file_exists()?;

        Ok(have_storage_db_files && have_key_file)
    }

    pub(crate) fn open_storage<'s>(&self, file_path: &str) -> Result<BazaFileStorage<'s>> {
        let key = self
            .key
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the key: {err}"))?;
        let key = key.as_ref().context("Key is missing")?;

        BazaStorage::read_file(file_path, key.clone())
    }

    fn merge_storages(&self) -> Result<()> {
        let db_files = self.paths.list_storage_db_files()?;

        if db_files.is_empty() {
            log::trace!("No existing db files found");
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

        let mut fs_tx = FsTransaction::new();

        // backup db files and open storages
        let storages = db_files
            .iter()
            .map(|db_file| {
                let new_db_file = fs_tx.move_to_backup(db_file)?;

                self.open_storage(&new_db_file)
                    .context(anyhow!("Failed to open storage for db {db_file}"))
            })
            .collect::<Result<Vec<_>>>()?;

        merge_storages_to_file(storages, main_db_file)?;

        fs_tx.commit()?;

        Ok(())
    }

    fn remove_unused_storage_blobs(&self, state: &BazaState) -> Result<()> {
        let blob_refs = state.get_all_blob_refs();
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

    fn update_state_from_storage(&self, state: &mut BazaState) -> Result<()> {
        let key = self
            .key
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the key: {err}"))?;
        let key = key.as_ref().context("Key is missing")?;

        let mut storage = BazaStorage::read_file(&self.paths.storage_main_db_file, key.clone())?;

        let latest_snapshots_count = update_state_from_storage(state, &mut storage)?;

        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");

            state.write_to_file(&self.paths.state_file, key.clone())?;
        }

        Ok(())
    }

    pub fn get_state_file_modification_time(&self) -> Result<Timestamp> {
        get_file_modification_time(&self.paths.state_file)
    }

    pub fn create(&self, password: SecretString) -> Result<()> {
        log::info!("Creating baza in {}", self.paths);

        self.paths.ensure_dirs_exist()?;

        ensure!(
            self.paths.list_storage_db_files()?.is_empty(),
            "Storage files already exist"
        );
        ensure!(!self.paths.key_file_exists()?, "Key file already exists");
        ensure!(
            !self.paths.state_file_exists()?,
            "State file already exists"
        );

        let _lock = LockFile::wait_for_lock(&self.paths.lock_file)?;

        let key_file_key = AgeKey::from_password(password)?;
        let key = self.generate_key_file(key_file_key)?;

        let info = BazaInfo {
            data_version: self.schema.get_latest_data_version(),
            storage_version: STORAGE_VERSION,
        };
        create_empty_storage_file(&self.paths.storage_main_db_file, key.clone(), &info)?;

        let mut key_guard = self
            .key
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock for the key: {err}"))?;
        key_guard.replace(key);

        log::info!(
            "Created new main storage file {}",
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

    pub fn unlock(&self, password: SecretString) -> Result<()> {
        log::debug!("Reading key file {}", self.paths.key_file);

        let key_file_key = AgeKey::from_password(password)?;

        let key = read_and_decrypt_file(&self.paths.key_file, key_file_key)?;

        let key = AgeKey::from_age_x25519_key(key.try_into()?)?;

        let mut key_guard = self
            .key
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock for the key: {err}"))?;
        key_guard.replace(key);

        Ok(())
    }

    pub fn lock(&self) -> Result<()> {
        let mut key_guard = self
            .key
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock for the key: {err}"))?;
        key_guard.take();

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

    pub fn create_state(&self, instance_id: InstanceId) -> Result<BazaState> {
        log::info!(
            "Creating new state file {} for instance {instance_id}",
            self.paths.state_file
        );

        ensure!(
            !self.paths.state_file_exists()?,
            "State file already exists"
        );

        let db_files = self.paths.list_storage_db_files()?;
        ensure!(!db_files.is_empty(), "No existing db files found");

        // Use main db file if exists, otherwise use the first in the list
        let mut db_file = &self.paths.storage_main_db_file;
        if !db_files.contains(db_file) {
            db_file = &db_files[0];
        }
        log::info!("Using {db_file} db file to create new state file");

        let key = self
            .key
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the key: {err}"))?;
        let key = key.as_ref().context("Key is missing")?;

        let mut storage = BazaStorage::read_file(db_file, key.clone())?;

        let info = storage.get_info()?;

        let mut state = BazaState::new(
            instance_id,
            info.clone(),
            self.schema.clone(),
            HashMap::new(),
        )?;
        state.write_to_file(&self.paths.state_file, key.clone())?;

        log::info!("Created new state file {}", self.paths.state_file);

        Ok(state)
    }

    pub fn dangerously_insert_snapshots_into_storage(
        &self,
        new_snapshots: &[Document],
    ) -> Result<()> {
        log::warn!(
            "Inserting {} documents into storage db {}",
            new_snapshots.len(),
            self.paths.storage_main_db_file
        );

        let _lock = LockFile::wait_for_lock(&self.paths.lock_file)?;

        let mut tx = FsTransaction::new();

        let old_db_file = tx.move_to_backup(self.paths.storage_main_db_file.clone())?;

        let storage = self.open_storage(&old_db_file)?;

        let patch = create_container_patch(new_snapshots.iter())?;
        storage.patch_and_save_to_file(&self.paths.storage_main_db_file, patch)?;

        tx.commit()?;

        Ok(())
    }

    pub fn dangerously_insert_blob_into_storage(&self, file_path: &str) -> Result<()> {
        log::warn!("Adding file {file_path} to storage");

        let _lock = LockFile::wait_for_lock(&self.paths.lock_file)?;

        ensure!(
            file_exists(file_path)?,
            "BLOB source must exist and must be a file"
        );

        let blob_id = BLOBId::from_file(file_path)?;
        let blob_path = self.paths.get_storage_blob_path(&blob_id);
        ensure!(!file_exists(&blob_path)?, "storage BLOB already exists");

        let key = self
            .key
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the key: {err}"))?;
        let key = key.as_ref().context("Key is missing")?;

        write_and_encrypt_blob(file_path, &blob_path, key.clone())?;

        Ok(())
    }

    #[cfg(test)]
    pub fn new_for_tests(test_dir: &str) -> Self {
        let schema = DataSchema::new_test_schema();

        let manager = BazaManager::new(
            format!("{test_dir}/storage"),
            format!("{test_dir}/state"),
            schema,
        );

        manager
            .create("test password".into())
            .expect("must create test baza");

        manager
    }

    pub fn get_schema(&self) -> &DataSchema {
        &self.schema
    }

    pub fn get_document_expert(&self) -> DocumentExpert {
        DocumentExpert::new(&self.schema)
    }

    pub fn get_state_dir(&self) -> &str {
        &self.paths.state_dir
    }

    pub fn is_locked(&self) -> bool {
        !self.is_unlocked()
    }

    pub fn is_unlocked(&self) -> bool {
        let key = self
            .key
            .read()
            .expect("Failed to acquire read lock for the key");

        key.is_some()
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

        assert!(dir_exists(&manager.paths.storage_dir).unwrap());
        assert!(dir_exists(&manager.paths.state_dir).unwrap());

        assert!(file_exists(&manager.paths.state_file).unwrap());
        assert!(file_exists(&manager.paths.storage_main_db_file).unwrap());

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
        assert!(manager.paths.storage_blob_exists(&blob1).unwrap());

        // ensure unused state BLOB is removed
        assert!(!manager.paths.storage_blob_exists(&blob2).unwrap());
        assert!(manager.paths.list_state_blobs().unwrap().is_empty());

        assert!(!baza.has_staged_documents());

        let storage = manager
            .open_storage(&manager.paths.storage_main_db_file)
            .unwrap();
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
        let storage = manager
            .open_storage(&manager.paths.storage_main_db_file)
            .unwrap();
        assert!(storage.contains(&doc_a1_key));
        assert!(manager.paths.storage_blob_exists(&blob_id).unwrap());

        // Erase the document and commit
        baza.erase_document(&doc_a1.id).unwrap();
        baza.commit().unwrap();
        drop(baza);

        // Reopen storage and check the snapshot and BLOB are removed
        let baza = manager.open().unwrap();
        let doc_a2_key = baza.get_document(&doc_a1.id).unwrap().create_key();
        let storage = manager
            .open_storage(&manager.paths.storage_main_db_file)
            .unwrap();
        assert!(!storage.contains(&doc_a1_key));
        assert!(storage.contains(&doc_a2_key));
        assert!(!manager.paths.storage_blob_exists(&blob_id).unwrap());
    }

    #[test]
    fn test_merge_storages() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open().unwrap();

        let db_file_1 = manager.paths.get_storage_file("db1");
        let db_file_2 = manager.paths.get_storage_file("db2");

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

        let storage = manager
            .open_storage(&manager.paths.storage_main_db_file)
            .unwrap();
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
            let manager = BazaManager::new(storage_dir, state_dir, schema);

            assert!(manager.open().is_err(), "Can't open without password");
            assert!(
                manager.unlock("wrong password".into()).is_err(),
                "Can't open with wrong password"
            );
            manager.unlock("test password".into()).unwrap();

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
            let manager = BazaManager::new(storage_dir, state_dir, schema);

            assert!(
                manager.unlock("test password".into()).is_err(),
                "Can't open with old password"
            );
            manager.unlock("new password".into()).unwrap();
        }
    }
}
