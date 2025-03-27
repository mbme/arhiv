mod keys;
mod manager_state;
mod migration;

use std::sync::RwLock;

use anyhow::{anyhow, ensure, Context, Result};

use rs_utils::{age::AgeKey, log, FsTransaction, LockFile, SecretString, Timestamp};

use crate::{schema::DataSchema, DocumentExpert};

use super::{
    baza_paths::BazaPaths,
    baza_storage::{create_empty_storage_file, merge_storages_to_file, STORAGE_VERSION},
    BazaInfo, BazaStorage,
};

use self::manager_state::BazaManagerState;

pub struct BazaManager {
    schema: DataSchema,
    pub(crate) paths: BazaPaths,
    state: RwLock<BazaManagerState>,
}

impl BazaManager {
    pub const MIN_PASSWORD_LENGTH: usize = AgeKey::MIN_PASSWORD_LEN;

    pub fn new(
        storage_dir: String,
        state_dir: String,
        downloads_dir: String,
        schema: DataSchema,
    ) -> Self {
        let paths = BazaPaths::new(storage_dir, state_dir, downloads_dir);

        BazaManager {
            schema,
            paths,
            state: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn clear_cached_baza(&self) -> Result<()> {
        let mut state = self.acquire_state_write_lock()?;
        state.clear_cached_baza();

        Ok(())
    }

    fn wait_for_file_lock(&self) -> Result<LockFile> {
        LockFile::wait_for_lock(&self.paths.lock_file)
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

    fn merge_storages(&self, key: &AgeKey) -> Result<()> {
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

                BazaStorage::read_file(&new_db_file, key.clone())
                    .context(anyhow!("Failed to open storage for db {db_file}"))
            })
            .collect::<Result<Vec<_>>>()?;

        merge_storages_to_file(storages, main_db_file)?;

        fs_tx.commit()?;

        Ok(())
    }

    pub fn get_state_file_modification_time(&self) -> Result<Timestamp> {
        self.paths.read_state_file_modification_time()
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

        let lock = self.wait_for_file_lock()?;
        let mut state = self.acquire_state_write_lock()?;

        let key_file_key = AgeKey::from_password(password)?;
        let key = self.generate_key_file(key_file_key, &lock)?;

        let info = BazaInfo {
            data_version: self.schema.get_latest_data_version(),
            storage_version: STORAGE_VERSION,
        };
        create_empty_storage_file(&self.paths.storage_main_db_file, key.clone(), &info)?;

        state.unlock(key);

        log::info!(
            "Created new main storage file {}",
            self.paths.storage_main_db_file
        );

        Ok(())
    }

    #[cfg(test)]
    pub fn new_for_tests(test_dir: &str) -> Self {
        let schema = DataSchema::new_test_schema();

        BazaManager::new_for_tests_with_schema(test_dir, schema)
    }

    #[cfg(test)]
    pub fn new_for_tests_with_schema(test_dir: &str, schema: DataSchema) -> Self {
        let manager = BazaManager::new(
            format!("{test_dir}/storage"),
            format!("{test_dir}/state"),
            format!("{test_dir}/downloads"),
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

    pub fn get_storage_dir(&self) -> &str {
        &self.paths.storage_dir
    }

    pub fn get_downloads_dir(&self) -> &str {
        &self.paths.storage_dir
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use rs_utils::{dir_exists, file_exists, TempFile};

    use crate::{
        baza2::{baza_manager::BazaManager, baza_storage::BazaFileStorage, BazaStorage},
        entities::{new_document, new_empty_document},
    };

    fn open_storage<'s>(manager: &BazaManager) -> BazaFileStorage<'s> {
        let key = manager
            .acquire_state_read_lock()
            .unwrap()
            .get_key()
            .unwrap()
            .clone();

        BazaStorage::read_file(&manager.paths.storage_main_db_file, key).unwrap()
    }

    #[test]
    fn test_commit() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open_mut().unwrap();

        assert!(dir_exists(&manager.paths.storage_dir).unwrap());
        assert!(dir_exists(&manager.paths.state_dir).unwrap());

        assert!(file_exists(&manager.paths.state_file).unwrap());
        assert!(file_exists(&manager.paths.storage_main_db_file).unwrap());

        let blob1_file = temp_dir.new_child("blob1");
        blob1_file.write_str("blob1").unwrap();

        let blob2_file = temp_dir.new_child("blob2");
        blob2_file.write_str("blob2").unwrap();

        assert!(
            baza.stage_document(new_document(json!({ "ref": "unknown" })), &None)
                .is_err(),
            "Can't stage document that references unknown document"
        );

        let asset1 = baza.create_asset(&blob1_file.path).unwrap();

        let asset2 = baza.create_asset(&blob2_file.path).unwrap();
        baza.erase_document(&asset2.id).unwrap();

        assert!(baza.has_staged_documents());

        baza.insert_snapshot(new_document(json!({})).with_rev(json!({ "a": 1 })))
            .unwrap();

        baza.commit().unwrap();
        drop(baza);

        let baza = manager.open().unwrap();

        // ensure new BLOB is committed
        assert!(manager.paths.storage_blob_exists(&asset1.id).unwrap());

        // ensure unused state BLOB is removed
        assert!(!manager.paths.storage_blob_exists(&asset2.id).unwrap());
        assert!(manager.paths.list_state_blobs().unwrap().is_empty());

        assert!(!baza.has_staged_documents());

        let storage = open_storage(&manager);
        assert_eq!(storage.index.len(), 3);
    }

    #[test]
    fn test_removes_erased_snapshots_from_storage() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open_mut().unwrap();

        // Create and stage a new document with a BLOB
        let blob_file = temp_dir.new_child("blob");
        blob_file.write_str("blob_content").unwrap();
        let asset_a1 = baza.create_asset(&blob_file.path).unwrap();
        baza.commit().unwrap();
        drop(baza);

        // Ensure the document and BLOB are in storage
        let baza = manager.open().unwrap();
        let doc_a1_key = baza.get_document(&asset_a1.id).unwrap().create_key();
        let storage = open_storage(&manager);
        assert!(storage.contains(&doc_a1_key));
        assert!(manager.paths.storage_blob_exists(&asset_a1.id).unwrap());
        drop(baza);

        // Erase the document and commit
        let mut baza = manager.open_mut().unwrap();
        baza.erase_document(&asset_a1.id).unwrap();
        baza.commit().unwrap();
        drop(baza);

        manager.clear_cached_baza().unwrap();

        // Reopen storage and check the snapshot and BLOB are removed
        let baza = manager.open().unwrap();
        let doc_a2_key = baza.get_document(&asset_a1.id).unwrap().create_key();
        let storage = open_storage(&manager);
        assert!(!storage.contains(&doc_a1_key));
        assert!(storage.contains(&doc_a2_key));
        assert!(!manager.paths.storage_blob_exists(&asset_a1.id).unwrap());
    }

    #[test]
    fn test_merge_storages() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open_mut().unwrap();

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

        manager.clear_cached_baza().unwrap();

        let baza = manager.open().unwrap();

        assert!(!file_exists(&db_file_1).unwrap());
        assert!(!file_exists(&db_file_2).unwrap());

        assert_eq!(baza.iter_documents().count(), 3);

        let storage = open_storage(&manager);
        assert_eq!(storage.index.len(), 4);
    }

    #[test]
    fn test_preserve_state() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);

        {
            let mut baza = manager.open_mut().unwrap();
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
        let downloads_dir = manager.paths.downloads_dir.clone();
        let schema = manager.schema.clone();

        {
            let mut baza = manager.open_mut().unwrap();
            let document = new_empty_document();
            baza.stage_document(document.clone(), &None).unwrap();
            baza.lock_document(&document.id, "test").unwrap();
            baza.save_changes().unwrap();
        }

        {
            let manager = BazaManager::new(storage_dir, state_dir, downloads_dir, schema);

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
}
