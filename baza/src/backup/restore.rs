use std::{collections::HashMap, fs, io::copy, path::Path};

use anyhow::{Context, Result, ensure};

use baza_common::{
    FsTransaction, LockFile, SecretString, TempFile, Timestamp, bytes_to_hex_string,
    create_file_reader, dir_exists, list_files, path_to_string,
};
use baza_storage::crypto::age::AgeKey;

use crate::{
    Baza, BazaManager, BazaPaths,
    entities::Id,
    schema::{ASSET_TYPE, Asset, AssetData},
};

use super::manifest::{BackupBlobArtifact, BackupBundle};

/// Restore behavior switches for backup validation and live application.
#[derive(Debug, Clone, Copy, Default)]
pub struct RestoreOptions {
    /// Permits restore when DB-referenced asset blobs are absent from the backup bundle.
    pub allow_missing_blobs: bool,
    /// Reads and decrypts every present referenced blob to verify plaintext size and SHA-256.
    pub deep: bool,
    /// Permits replacing newer live storage with an older backup.
    pub allow_rollback: bool,
}

/// Summary of restore preflight work performed for a backup manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreCheckReport {
    /// Manifest path supplied by the caller.
    pub manifest_path: String,
    /// Backup timestamp recorded in the manifest.
    pub timestamp: String,
    /// Number of asset blobs referenced by the backed-up DB.
    pub referenced_blobs: usize,
    /// Referenced asset IDs that have no present blob artifact.
    pub missing_blobs: Vec<Id>,
    /// Number of key, DB, and blob ciphertext artifacts verified against the manifest.
    pub verified_artifacts: usize,
    /// Number of blob plaintext streams verified by `--deep`.
    pub deep_verified_blobs: usize,
}

struct StorageDbInfo {
    assets: HashMap<Id, AssetData>,
    last_modification_time: Option<Timestamp>,
}

struct LiveRestoreInfo {
    has_staged_documents: bool,
    last_modification_time: Option<Timestamp>,
}

struct CheckedBackup {
    backup: BackupBundle,
    db_info: StorageDbInfo,
    report: RestoreCheckReport,
}

impl BazaManager {
    /// Performs read-only restore preflight for an encrypted backup manifest.
    ///
    /// Authenticates the manifest, verifies listed ciphertext artifacts, and opens the backed-up
    /// DB to check its blob references. With [`RestoreOptions::deep`], it also verifies present
    /// referenced blob plaintext against DB metadata. This does not mutate live storage.
    pub fn restore_check(
        &self,
        manifest_path: &str,
        password: SecretString,
        options: RestoreOptions,
    ) -> Result<RestoreCheckReport> {
        Ok(self
            .load_checked_backup(manifest_path, password, options)?
            .report)
    }

    fn load_checked_backup(
        &self,
        manifest_path: &str,
        password: SecretString,
        options: RestoreOptions,
    ) -> Result<CheckedBackup> {
        let backup = BackupBundle::load(manifest_path, password)?;

        backup.verify_key_and_db()?;
        let db_info = self.read_backup_db_info(&backup)?;

        let mut blob_artifacts = HashMap::new();
        for blob in backup.blobs() {
            if backup.blob_exists(blob)? {
                backup.verify_blob(blob)?;
                blob_artifacts.insert(blob.id.clone(), blob);
            }
        }
        let mut missing_blobs = db_info
            .assets
            .keys()
            .filter(|id| !blob_artifacts.contains_key(*id))
            .cloned()
            .collect::<Vec<_>>();
        missing_blobs.sort_by_key(ToString::to_string);

        ensure!(
            options.allow_missing_blobs || missing_blobs.is_empty(),
            "Backup is missing {} referenced BLOBs",
            missing_blobs.len()
        );

        let mut deep_verified_blobs = 0;
        if options.deep {
            deep_verified_blobs =
                self.deep_verify_backup_blobs(&backup, &db_info, &blob_artifacts)?;
        }

        let report = RestoreCheckReport {
            manifest_path: manifest_path.to_string(),
            timestamp: backup.timestamp().to_string(),
            referenced_blobs: db_info.assets.len(),
            missing_blobs,
            verified_artifacts: 2 + blob_artifacts.len(),
            deep_verified_blobs,
        };

        Ok(CheckedBackup {
            backup,
            db_info,
            report,
        })
    }

    /// Replaces live storage with a validated backup generation and clears runtime state.
    ///
    /// Requires unlocked live storage and exclusive access. Refuses staged live changes and,
    /// unless explicitly allowed, rollback over newer live data. File changes participate in a
    /// filesystem transaction, the restored DB is validated before commit, and the manager ends
    /// locked so runtime state can be regenerated on the next open.
    pub fn restore_apply(
        &self,
        manifest_path: &str,
        password: SecretString,
        options: RestoreOptions,
    ) -> Result<RestoreCheckReport> {
        self.paths.ensure_dirs_exist()?;
        let live_storage_key = AgeKey::from_age_x25519_key(self.get_unlocked_storage_key()?)
            .context("Restore apply safety checks require current Arhiv storage to be unlocked")?;

        let CheckedBackup {
            backup,
            db_info,
            report,
        } = self.load_checked_backup(manifest_path, password, options)?;

        let _lock = LockFile::must_lock(&self.paths.lock_file)
            .context("Restore requires exclusive access to Arhiv storage")?;

        let live_info = self.read_live_restore_info(&live_storage_key)?;
        ensure!(
            !live_info.has_staged_documents,
            "Can't restore: current Arhiv has staged changes"
        );
        ensure!(
            options.allow_rollback
                || !is_live_newer_than_backup(
                    live_info.last_modification_time,
                    db_info.last_modification_time,
                ),
            "Can't restore older backup over newer current storage without explicit rollback permission"
        );

        let mut fs_tx = FsTransaction::new();

        backup.copy_key_to(&mut fs_tx, &self.paths.key_file)?;
        backup.copy_db_to(&mut fs_tx, &self.paths.storage_main_db_file)?;

        fs_tx.create_dir_if_missing(&self.paths.storage_data_dir)?;
        for blob in backup.blobs() {
            let dest = self.paths.get_storage_blob_path(&blob.id);
            if !backup.blob_exists(blob)? {
                ensure!(
                    report.missing_blobs.contains(&blob.id),
                    "Backup artifact for BLOB {} is missing",
                    blob.id
                );
                fs_tx
                    .remove_file_if_exists(&dest)
                    .with_context(|| format!("Failed to remove stale live BLOB {}", blob.id))?;
                continue;
            }
            backup
                .copy_blob_to(&mut fs_tx, blob, &dest)
                .with_context(|| format!("Failed to restore BLOB {}", blob.id))?;
        }

        self.clear_runtime_state(&mut fs_tx)?;
        self.read_storage_db_info(
            Path::new(&self.paths.storage_main_db_file),
            backup.storage_key(),
        )
        .context("Failed to validate restored storage DB")?;
        fs_tx.commit()?;
        self.lock()?;

        Ok(report)
    }

    fn read_backup_db_info(&self, backup: &BackupBundle) -> Result<StorageDbInfo> {
        self.read_storage_db_info(&backup.db_path()?, backup.storage_key())
    }

    fn read_storage_db_info(&self, db_src: &Path, storage_key: &AgeKey) -> Result<StorageDbInfo> {
        let temp_dir = TempFile::new_with_details("arhiv-restore-check-", "");
        temp_dir.mkdir()?;

        let paths = BazaPaths::new(
            format!("{}/storage", temp_dir.path),
            format!("{}/state", temp_dir.path),
            format!("{}/downloads", temp_dir.path),
        );
        paths.ensure_dirs_exist()?;

        fs::copy(db_src, &paths.storage_main_db_file).context("Failed to stage backup DB")?;

        let manager = BazaManager::new(paths, self.get_schema().clone());
        let serialized_key = storage_key.serialize();
        manager.unlock_using_storage_key(serialized_key)?;
        let baza = manager.open()?;

        Ok(StorageDbInfo {
            assets: current_asset_data_by_id(&baza)?,
            last_modification_time: baza.find_last_modification_time(),
        })
    }

    fn read_live_restore_info(&self, live_storage_key: &AgeKey) -> Result<LiveRestoreInfo> {
        let has_staged_documents = if self.paths.state_file_exists()? {
            Baza::read(
                live_storage_key.clone(),
                self.paths.clone(),
                self.get_schema().clone(),
            )?
            .has_staged_documents()
        } else {
            false
        };

        let storage_info = self.read_storage_db_info(
            Path::new(&self.paths.storage_main_db_file),
            live_storage_key,
        )?;

        Ok(LiveRestoreInfo {
            has_staged_documents,
            last_modification_time: storage_info.last_modification_time,
        })
    }

    fn deep_verify_backup_blobs(
        &self,
        backup: &BackupBundle,
        db_info: &StorageDbInfo,
        blob_artifacts: &HashMap<Id, &BackupBlobArtifact>,
    ) -> Result<usize> {
        let mut verified = 0;

        for (asset_id, asset_data) in &db_info.assets {
            let Some(blob_artifact) = blob_artifacts.get(asset_id) else {
                continue;
            };

            verify_asset_blob_plaintext(backup, blob_artifact, asset_data)?;
            verified += 1;
        }

        Ok(verified)
    }

    fn clear_runtime_state(&self, fs_tx: &mut FsTransaction) -> Result<()> {
        fs_tx.remove_file_if_exists(&self.paths.state_file)?;
        fs_tx.remove_file_if_exists(&self.paths.state_search_index_file)?;
        fs_tx.remove_file_if_exists(&self.paths.state_document_locks_file)?;

        if dir_exists(&self.paths.state_data_dir)? {
            for state_blob in list_files(&self.paths.state_data_dir)? {
                fs_tx.remove_file(state_blob)?;
            }
        } else {
            fs_tx.create_dir(&self.paths.state_data_dir)?;
        }

        Ok(())
    }
}

fn current_asset_data_by_id(baza: &Baza) -> Result<HashMap<Id, AssetData>> {
    let mut assets = HashMap::new();
    for head in baza.iter_documents() {
        if *head.get_type() != ASSET_TYPE {
            continue;
        }

        let document = head.get_single_document();
        if document.is_erased() {
            continue;
        }

        let asset: Asset = document.clone().convert()?;
        assets.insert(asset.id, asset.data);
    }

    Ok(assets)
}

fn verify_asset_blob_plaintext(
    backup: &BackupBundle,
    blob_artifact: &BackupBlobArtifact,
    asset_data: &AssetData,
) -> Result<()> {
    let path = backup.blob_path(blob_artifact)?;
    let blob_key = AgeKey::from_age_x25519_key(asset_data.age_x25519_key.clone())?;
    let mut reader = baza_storage::crypto::age::AgeReader::new(
        create_file_reader(&path_to_string(&path))?,
        blob_key,
    )?;
    let mut hashing_reader = baza_common::Sha256HashingReader::new(&mut reader);
    let size = copy(&mut hashing_reader, &mut std::io::sink())?;
    let hash = hashing_reader
        .get_hash()
        .context("Blob hash must be finalized after full read")?;
    let hash = bytes_to_hex_string(hash);

    ensure!(
        size == asset_data.size,
        "BLOB {} size mismatch",
        blob_artifact.id
    );
    ensure!(
        hash == asset_data.content_sha256,
        "BLOB {} plaintext SHA-256 mismatch",
        blob_artifact.id
    );

    Ok(())
}

fn is_live_newer_than_backup(
    live_last_modification_time: Option<Timestamp>,
    backup_last_modification_time: Option<Timestamp>,
) -> bool {
    match (live_last_modification_time, backup_last_modification_time) {
        (Some(live), Some(backup)) => live > backup,
        (Some(_), None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baza_common::{TempFile, dir_exists, file_exists};

    use crate::BazaManager;

    use super::super::manifest::MANIFEST_SUFFIX;

    use super::*;

    fn create_committed_asset(
        manager: &BazaManager,
        temp_dir: &TempFile,
        name: &str,
    ) -> Result<Id> {
        let file = temp_dir.new_child(name);
        file.write_str(format!("content for {name}"))?;

        let mut baza = manager.open_mut()?;
        let asset = baza.create_asset(&file.path)?;
        baza.commit()?;

        Ok(asset.id)
    }

    fn create_backup(manager: &BazaManager, temp_dir: &TempFile) -> Result<String> {
        let backup_dir = format!("{}/backup", temp_dir.path);
        fs::create_dir(&backup_dir)?;

        manager.backup(&backup_dir)?;

        let manifests = list_files(&backup_dir)?
            .into_iter()
            .filter(|path| path.ends_with(MANIFEST_SUFFIX))
            .collect::<Vec<_>>();
        ensure!(manifests.len() == 1, "expected one manifest");

        Ok(manifests[0].clone())
    }

    #[test]
    fn restore_check_validates_manifest_backup() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_restore", "");
        temp_dir.mkdir()?;
        let manager = BazaManager::new_for_tests(&temp_dir.path);
        create_committed_asset(&manager, &temp_dir, "asset-one")?;

        let manifest_path = create_backup(&manager, &temp_dir)?;
        let report = manager.restore_check(
            &manifest_path,
            "test password".into(),
            RestoreOptions::default(),
        )?;

        assert_eq!(report.referenced_blobs, 1);
        assert!(report.missing_blobs.is_empty());
        assert_eq!(report.verified_artifacts, 3);

        Ok(())
    }

    #[test]
    fn restore_check_allows_missing_blobs_only_when_explicit() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_restore", "");
        temp_dir.mkdir()?;
        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let asset_id = create_committed_asset(&manager, &temp_dir, "asset-one")?;

        let manifest_path = create_backup(&manager, &temp_dir)?;
        let backup_dir = Path::new(&manifest_path)
            .parent()
            .expect("manifest has parent");
        fs::remove_file(backup_dir.join(format!("data/{asset_id}.age")))?;

        assert!(
            manager
                .restore_check(
                    &manifest_path,
                    "test password".into(),
                    RestoreOptions::default(),
                )
                .is_err()
        );

        let report = manager.restore_check(
            &manifest_path,
            "test password".into(),
            RestoreOptions {
                allow_missing_blobs: true,
                deep: false,
                allow_rollback: false,
            },
        )?;
        assert_eq!(report.missing_blobs, vec![asset_id]);
        assert_eq!(report.verified_artifacts, 2);

        Ok(())
    }

    #[test]
    fn restore_apply_restores_older_db_and_clears_runtime_state() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_restore", "");
        temp_dir.mkdir()?;
        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let first_asset_id = create_committed_asset(&manager, &temp_dir, "asset-one")?;
        let manifest_path = create_backup(&manager, &temp_dir)?;

        let second_asset_id = create_committed_asset(&manager, &temp_dir, "asset-two")?;
        assert!(manager.open()?.get_asset(&second_asset_id)?.is_some());

        let err = manager
            .restore_apply(
                &manifest_path,
                "test password".into(),
                RestoreOptions::default(),
            )
            .expect_err("restore apply must reject rollback by default");
        assert!(err.to_string().contains("rollback permission"));

        manager.restore_apply(
            &manifest_path,
            "test password".into(),
            RestoreOptions {
                allow_rollback: true,
                ..RestoreOptions::default()
            },
        )?;

        assert!(manager.is_locked());
        assert!(!file_exists(&manager.paths.state_file)?);
        assert!(dir_exists(&manager.paths.state_data_dir)?);

        manager.unlock("test password".into())?;
        let baza = manager.open()?;
        assert!(baza.get_asset(&first_asset_id)?.is_some());
        assert!(baza.get_asset(&second_asset_id)?.is_none());

        Ok(())
    }

    #[test]
    fn restore_apply_removes_live_blob_when_backup_allows_missing_blob() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_restore", "");
        temp_dir.mkdir()?;
        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let asset_id = create_committed_asset(&manager, &temp_dir, "asset-one")?;

        let manifest_path = create_backup(&manager, &temp_dir)?;
        let backup_dir = Path::new(&manifest_path)
            .parent()
            .expect("manifest has parent");
        fs::remove_file(backup_dir.join(format!("data/{asset_id}.age")))?;

        let live_blob_path = manager.paths.get_storage_blob_path(&asset_id);
        assert!(file_exists(&live_blob_path)?);

        manager.restore_apply(
            &manifest_path,
            "test password".into(),
            RestoreOptions {
                allow_missing_blobs: true,
                ..RestoreOptions::default()
            },
        )?;

        assert!(!file_exists(&live_blob_path)?);

        Ok(())
    }

    #[test]
    fn restore_apply_rejects_staged_changes() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_restore", "");
        temp_dir.mkdir()?;
        let manager = BazaManager::new_for_tests(&temp_dir.path);
        create_committed_asset(&manager, &temp_dir, "asset-one")?;
        let manifest_path = create_backup(&manager, &temp_dir)?;

        let file = temp_dir.new_child("asset-two");
        file.write_str("staged asset content")?;
        {
            let mut baza = manager.open_mut()?;
            baza.create_asset(&file.path)?;
            baza.save_changes()?;
        }

        let err = manager
            .restore_apply(
                &manifest_path,
                "test password".into(),
                RestoreOptions {
                    allow_rollback: true,
                    ..RestoreOptions::default()
                },
            )
            .expect_err("restore apply must reject staged changes");
        assert!(err.to_string().contains("staged changes"));

        Ok(())
    }
}
