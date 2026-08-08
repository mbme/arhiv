use std::{collections::HashMap, io::Write, time::Instant};

use anyhow::{Context, Result, ensure};

use baza_common::{
    FsTransaction, bytes_to_hex_string, create_file_reader, create_file_writer, file_exists,
    generate_alpanumeric_string, get_file_hash_sha256, log,
};
use baza_storage::crypto::age::{AgeKey, AgeReader};

use crate::{
    BazaInfo, BazaState, BazaStorage,
    baza_manager::manager_state::BazaManagerState,
    baza_storage::{STORAGE_VERSION, create_storage},
    entities::{Document, Id},
    schema::ASSET_TYPE,
};

use super::super::BazaManager;

const SOURCE_DATA_VERSION: u8 = 1;
const TARGET_DATA_VERSION: u8 = 2;

impl BazaManager {
    /// Migrates data version 1 storage to data version 2 by backfilling asset plaintext hashes.
    ///
    /// The migration requires a clean local state and an unlocked storage key because every asset
    /// blob is decrypted to compute its canonical plaintext SHA-256 digest. Returns `true` when
    /// storage or local state artifacts changed, and `false` when storage was already clean v2.
    pub fn migrate_data_v1_to_v2_asset_content_sha256(&self) -> Result<bool> {
        ensure!(self.storage_exists()?, "Storage doesn't exist");
        self.paths.ensure_dirs_exist()?;

        let _lock = self.wait_for_file_lock()?;
        let mut manager_state = self.acquire_state_write_lock()?;

        let key = manager_state.get_key()?.clone();
        self.migrate_data_v1_to_v2_asset_content_sha256_with_state(&mut manager_state, &key)
    }

    pub(super) fn migrate_data_v1_to_v2_asset_content_sha256_with_state(
        &self,
        manager_state: &mut BazaManagerState,
        key: &AgeKey,
    ) -> Result<bool> {
        log::info!(
            "Preparing data migration from version {SOURCE_DATA_VERSION} to {TARGET_DATA_VERSION}"
        );

        self.recover_main_storage_db()?;
        let local_state = BazaState::read_local_migration_status(&self.paths, key.clone())
            .context("Failed to inspect local state before migration")?;
        let db_files = self.paths.list_storage_db_files()?;
        ensure!(!db_files.is_empty(), "No existing db files found");

        let source_info = self.read_common_storage_info(&db_files, key.clone())?;
        ensure!(
            source_info.storage_version == STORAGE_VERSION,
            "Storage version mismatch: expected {}, got {}",
            STORAGE_VERSION,
            source_info.storage_version
        );

        if source_info.data_version == TARGET_DATA_VERSION {
            let removed_state = if local_state.data_version != Some(TARGET_DATA_VERSION) {
                self.ensure_local_state_can_migrate(&local_state)?;
                self.remove_local_state_artifacts_transactionally()?
            } else {
                false
            };
            manager_state.clear_cached_baza();
            log::info!("Storage is already at data version {TARGET_DATA_VERSION}");
            return Ok(removed_state);
        }

        ensure!(
            source_info.data_version == SOURCE_DATA_VERSION,
            "Unsupported data migration path: {} to {}",
            source_info.data_version,
            TARGET_DATA_VERSION
        );

        self.ensure_local_state_can_migrate(&local_state)?;

        let started_at = Instant::now();
        let mut asset_hashes = HashMap::new();
        let migrated_files = db_files
            .iter()
            .map(|db_file| {
                self.write_migrated_asset_hash_storage(db_file, key.clone(), &mut asset_hashes)
            })
            .collect::<Result<Vec<_>>>()?;

        let mut fs_tx = FsTransaction::new();
        for (db_file, migrated_file) in db_files.iter().zip(migrated_files.iter()) {
            fs_tx.move_file(migrated_file, db_file, false)?;
        }
        self.remove_local_state_artifacts(&mut fs_tx)?;
        fs_tx.commit()?;

        manager_state.clear_cached_baza();

        log::info!(
            "Migrated {} storage file(s) from data version {} to {} in {:?}",
            db_files.len(),
            SOURCE_DATA_VERSION,
            TARGET_DATA_VERSION,
            started_at.elapsed()
        );

        Ok(true)
    }

    fn ensure_local_state_can_migrate(
        &self,
        local_state: &crate::baza_state::LocalStateMigrationStatus,
    ) -> Result<()> {
        ensure!(
            !local_state.has_staged_documents,
            "Can't migrate while local state has staged documents. Open this storage with the previous Arhiv version, commit or reset local changes, then upgrade again."
        );
        ensure!(
            !local_state.has_document_locks,
            "Can't migrate while local state has document locks. Open this storage with the previous Arhiv version, clear locks or reset local state, then upgrade again."
        );

        let state_blobs = self.paths.list_state_blobs()?;
        ensure!(
            state_blobs.is_empty(),
            "Can't migrate while local state has {} blob(s). Open this storage with the previous Arhiv version, commit or reset local asset changes, then upgrade again.",
            state_blobs.len()
        );

        Ok(())
    }

    fn read_common_storage_info(&self, db_files: &[String], key: AgeKey) -> Result<BazaInfo> {
        let mut infos = db_files
            .iter()
            .map(|db_file| {
                let mut storage = BazaStorage::read_file(db_file, key.clone())
                    .with_context(|| format!("Failed to read storage info from {db_file}"))?;

                storage.get_info().cloned()
            })
            .collect::<Result<Vec<_>>>()?;

        let first_info = infos.pop().context("No existing db files found")?;
        for info in infos {
            ensure!(
                info == first_info,
                "Can't migrate storage files with mixed BazaInfo values"
            );
        }

        Ok(first_info)
    }

    fn write_migrated_asset_hash_storage(
        &self,
        db_file: &str,
        key: AgeKey,
        asset_hashes: &mut HashMap<(Id, String), String>,
    ) -> Result<String> {
        log::info!("Migrating asset metadata in storage file {db_file}");

        let mut storage = BazaStorage::read_file(db_file, key.clone())?;
        let source_info = storage.get_info()?.clone();
        ensure!(
            source_info.data_version == SOURCE_DATA_VERSION,
            "Storage file {db_file} has data version {}, expected {}",
            source_info.data_version,
            SOURCE_DATA_VERSION
        );

        let expected_documents = storage.index.len();
        let mut documents = Vec::with_capacity(expected_documents);
        let mut migrated_assets = 0usize;

        while let Some(item) = storage.next_parsed() {
            let (key, mut document) = item?;
            ensure!(
                key == document.create_key(),
                "Storage key {} doesn't match document key {}",
                key.serialize(),
                document.create_key().serialize()
            );

            if document.document_type.is(ASSET_TYPE) {
                let content_sha256 =
                    self.compute_or_get_asset_content_sha256(&document, asset_hashes)?;
                document.data.set("content_sha256", content_sha256);
                migrated_assets += 1;
            }

            documents.push(document);
        }

        ensure!(
            documents.len() == expected_documents,
            "Storage file {db_file} yielded {} documents, expected {}",
            documents.len(),
            expected_documents
        );

        let target_info = BazaInfo {
            storage_version: source_info.storage_version,
            data_version: TARGET_DATA_VERSION,
        };
        let migrated_file = self.new_migration_temp_file(db_file);
        let mut writer = create_file_writer(&migrated_file, false)
            .with_context(|| format!("Failed to create migrated storage file {migrated_file}"))?;
        create_storage(&mut writer, key, target_info, &documents)
            .with_context(|| format!("Failed to write migrated storage file {migrated_file}"))?;
        writer.flush()?;

        log::info!(
            "Migrated {} asset snapshot(s) in storage file {db_file}",
            migrated_assets
        );

        Ok(migrated_file)
    }

    fn compute_or_get_asset_content_sha256(
        &self,
        document: &Document,
        asset_hashes: &mut HashMap<(Id, String), String>,
    ) -> Result<String> {
        let blob_key_string = document
            .data
            .get_mandatory_str("age_x25519_key")
            .to_string();
        let cache_key = (document.id.clone(), blob_key_string.clone());

        if let Some(content_sha256) = asset_hashes.get(&cache_key) {
            return Ok(content_sha256.clone());
        }

        let blob_key = AgeKey::from_age_x25519_key(blob_key_string.into())?;
        let blob_path = self.paths.get_storage_blob_path(&document.id);
        ensure!(
            file_exists(&blob_path)?,
            "Asset blob {} is missing from storage",
            document.id
        );

        let reader = create_file_reader(&blob_path)
            .with_context(|| format!("Failed to open asset blob {}", document.id))?;
        let age_reader = AgeReader::new(reader, blob_key)
            .with_context(|| format!("Failed to decrypt asset blob {}", document.id))?;
        let hash = get_file_hash_sha256(age_reader)
            .with_context(|| format!("Failed to hash asset blob {}", document.id))?;

        let content_sha256 = bytes_to_hex_string(&hash);
        asset_hashes.insert(cache_key, content_sha256.clone());

        Ok(content_sha256)
    }

    fn new_migration_temp_file(&self, db_file: &str) -> String {
        format!(
            "{db_file}.v{SOURCE_DATA_VERSION}-to-v{TARGET_DATA_VERSION}-{}.tmp",
            generate_alpanumeric_string(10)
        )
    }

    fn remove_local_state_artifacts_transactionally(&self) -> Result<bool> {
        let mut fs_tx = FsTransaction::new();
        let removed = self.remove_local_state_artifacts(&mut fs_tx)?;
        fs_tx.commit()?;

        Ok(removed)
    }

    fn remove_local_state_artifacts(&self, fs_tx: &mut FsTransaction) -> Result<bool> {
        let mut removed = false;
        for file in [
            &self.paths.state_file,
            &self.paths.state_search_index_file,
            &self.paths.state_document_locks_file,
        ] {
            if file_exists(file)? {
                fs_tx.remove_file(file)?;
                removed = true;
            }
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::json;

    use baza_common::{
        TempFile, bytes_to_hex_string, create_file_writer, get_file_hash_sha256, read_all_as_string,
    };
    use baza_storage::crypto::age::AgeKey;

    use crate::{
        BazaInfo, BazaStorage,
        baza_storage::create_storage,
        entities::{Document, Id, new_empty_document},
        schema::ASSET_TYPE,
    };

    use super::{BazaManager, SOURCE_DATA_VERSION, STORAGE_VERSION, TARGET_DATA_VERSION};

    fn storage_key(manager: &BazaManager) -> AgeKey {
        manager
            .acquire_state_read_lock()
            .unwrap()
            .get_key()
            .unwrap()
            .clone()
    }

    fn read_main_storage_documents(manager: &BazaManager) -> Vec<Document> {
        BazaStorage::read_file(&manager.paths.storage_main_db_file, storage_key(manager))
            .unwrap()
            .get_all()
            .unwrap()
    }

    fn read_main_storage_info(manager: &BazaManager) -> BazaInfo {
        let mut storage =
            BazaStorage::read_file(&manager.paths.storage_main_db_file, storage_key(manager))
                .unwrap();

        storage.get_info().unwrap().clone()
    }

    fn rewrite_main_storage(manager: &BazaManager, info: BazaInfo, documents: &[Document]) {
        let mut writer = create_file_writer(&manager.paths.storage_main_db_file, true).unwrap();
        create_storage(&mut writer, storage_key(manager), info, documents).unwrap();
    }

    fn create_v1_asset_storage(manager: &BazaManager, asset_data: &str) -> (Id, String) {
        let source_file = TempFile::new();
        source_file.write_str(asset_data).unwrap();

        let asset = {
            let mut baza = manager.open_mut().unwrap();
            let asset = baza.create_asset(&source_file.path).unwrap();
            baza.commit().unwrap();
            asset
        };

        let expected_hash =
            bytes_to_hex_string(&get_file_hash_sha256(asset_data.as_bytes()).unwrap());
        let mut documents = read_main_storage_documents(manager);
        for document in &mut documents {
            if document.document_type.is(ASSET_TYPE) {
                document.data.remove("content_sha256");
            }
        }

        rewrite_main_storage(
            manager,
            BazaInfo {
                storage_version: STORAGE_VERSION,
                data_version: SOURCE_DATA_VERSION,
            },
            &documents,
        );

        (asset.id, expected_hash)
    }

    #[test]
    fn test_migrates_v1_asset_content_sha256() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_migrate_asset_hash", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let (asset_id, expected_hash) = create_v1_asset_storage(&manager, "asset data");

        assert!(manager.migrate_data_v1_to_v2_asset_content_sha256()?);

        assert_eq!(
            read_main_storage_info(&manager),
            BazaInfo {
                storage_version: STORAGE_VERSION,
                data_version: TARGET_DATA_VERSION,
            }
        );

        let asset = read_main_storage_documents(&manager)
            .into_iter()
            .find(|document| document.id == asset_id)
            .expect("asset document exists");

        assert_eq!(
            asset.data.get_mandatory_str("content_sha256"),
            expected_hash
        );

        let baza = manager.open()?;
        let migrated_asset = baza.get_asset(&asset_id)?.expect("asset exists");
        assert_eq!(migrated_asset.data.content_sha256, expected_hash);

        Ok(())
    }

    #[test]
    fn test_migrates_all_asset_snapshots() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_migrate_asset_snapshots", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let source_file = temp_dir.new_child("asset");
        source_file.write_str("asset data")?;

        let asset_id = {
            let mut baza = manager.open_mut()?;
            let asset = baza.create_asset(&source_file.path)?;
            baza.commit()?;
            asset.id
        };

        let expected_hash = bytes_to_hex_string(&get_file_hash_sha256("asset data".as_bytes())?);
        let mut documents = read_main_storage_documents(&manager);
        let asset_document = documents
            .iter()
            .find(|document| document.id == asset_id)
            .cloned()
            .expect("asset document exists");

        documents.push(asset_document.with_rev(json!({ "history": 1 })));
        for document in &mut documents {
            if document.document_type.is(ASSET_TYPE) {
                document.data.remove("content_sha256");
            }
        }

        rewrite_main_storage(
            &manager,
            BazaInfo {
                storage_version: STORAGE_VERSION,
                data_version: SOURCE_DATA_VERSION,
            },
            &documents,
        );

        assert!(manager.migrate_data_v1_to_v2_asset_content_sha256()?);

        let asset_snapshots = read_main_storage_documents(&manager)
            .into_iter()
            .filter(|document| document.id == asset_id)
            .collect::<Vec<_>>();
        assert_eq!(asset_snapshots.len(), 2);
        assert!(
            asset_snapshots
                .iter()
                .all(|document| document.data.get_mandatory_str("content_sha256") == expected_hash)
        );

        Ok(())
    }

    #[test]
    fn test_open_auto_migrates_v1_asset_content_sha256() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_auto_migrate_asset_hash", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let (asset_id, expected_hash) = create_v1_asset_storage(&manager, "asset data");
        manager.clear_cached_baza()?;

        let baza = manager.open()?;
        let migrated_asset = baza.get_asset(&asset_id)?.expect("asset exists");

        assert_eq!(migrated_asset.data.content_sha256, expected_hash);
        assert_eq!(
            read_main_storage_info(&manager).data_version,
            TARGET_DATA_VERSION
        );

        Ok(())
    }

    #[test]
    fn test_open_auto_migration_refuses_dirty_local_state() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_auto_migrate_dirty_state", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let source_file = temp_dir.new_child("asset");
        source_file.write_str("asset data")?;

        {
            let mut baza = manager.open_mut()?;
            baza.create_asset(&source_file.path)?;
            baza.commit()?;
            baza.stage_document(new_empty_document(), &None)?;
            baza.save_changes()?;
        }

        let mut documents = read_main_storage_documents(&manager);
        for document in &mut documents {
            if document.document_type.is(ASSET_TYPE) {
                document.data.remove("content_sha256");
            }
        }
        rewrite_main_storage(
            &manager,
            BazaInfo {
                storage_version: STORAGE_VERSION,
                data_version: SOURCE_DATA_VERSION,
            },
            &documents,
        );
        manager.clear_cached_baza()?;

        let err = match manager.open() {
            Ok(_) => panic!("dirty state blocks migration"),
            Err(err) => err,
        };
        assert!(
            format!("{err:#}").contains("previous Arhiv version"),
            "unexpected error: {err:#}"
        );
        assert_eq!(
            read_main_storage_info(&manager).data_version,
            SOURCE_DATA_VERSION
        );

        Ok(())
    }

    #[test]
    fn test_open_allows_dirty_local_state_when_storage_is_current() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_open_dirty_current_state", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        {
            let mut baza = manager.open_mut()?;
            baza.stage_document(new_empty_document(), &None)?;
            baza.save_changes()?;
        }

        manager.clear_cached_baza()?;
        let baza = manager.open()?;

        assert!(baza.has_staged_documents());
        assert_eq!(
            read_main_storage_info(&manager).data_version,
            TARGET_DATA_VERSION
        );

        Ok(())
    }

    #[test]
    fn test_migration_fails_when_asset_blob_is_missing() {
        let temp_dir = TempFile::new_with_details("baza_migrate_missing_blob", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let (asset_id, _) = create_v1_asset_storage(&manager, "asset data");
        std::fs::remove_file(manager.paths.get_storage_blob_path(&asset_id)).unwrap();

        assert!(
            manager
                .migrate_data_v1_to_v2_asset_content_sha256()
                .is_err()
        );
        assert_eq!(
            read_main_storage_info(&manager).data_version,
            SOURCE_DATA_VERSION
        );
    }

    #[test]
    fn test_migration_fails_when_asset_blob_is_corrupt() {
        let temp_dir = TempFile::new_with_details("baza_migrate_corrupt_blob", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let (asset_id, _) = create_v1_asset_storage(&manager, "asset data");
        std::fs::write(manager.paths.get_storage_blob_path(&asset_id), b"corrupt").unwrap();

        assert!(
            manager
                .migrate_data_v1_to_v2_asset_content_sha256()
                .is_err()
        );
        assert_eq!(
            read_main_storage_info(&manager).data_version,
            SOURCE_DATA_VERSION
        );
    }

    #[test]
    fn test_migration_is_idempotent_for_v2_storage() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_migrate_already_v2", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);

        assert!(!manager.migrate_data_v1_to_v2_asset_content_sha256()?);
        assert_eq!(
            read_main_storage_info(&manager).data_version,
            TARGET_DATA_VERSION
        );

        Ok(())
    }

    #[test]
    fn test_new_asset_records_content_sha256() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_asset_hash", "");
        temp_dir.mkdir()?;

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let source_file = temp_dir.new_child("asset");
        source_file.write_str("asset data")?;

        let asset = {
            let mut baza = manager.open_mut()?;
            baza.create_asset(&source_file.path)?
        };

        assert_eq!(
            asset.data.content_sha256,
            bytes_to_hex_string(&get_file_hash_sha256("asset data".as_bytes())?)
        );

        let baza = manager.open()?;
        let decrypted = read_all_as_string(baza.get_asset_data(&asset.id)?)?;
        assert_eq!(decrypted, "asset data");

        Ok(())
    }
}
