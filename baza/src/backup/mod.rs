use std::path::Path;

use anyhow::{Result, bail, ensure};

use baza_common::{
    FsTransaction, Timestamp, create_dir_if_not_exist, ensure_dir_exists, file_exists,
    get_file_name, list_files, log,
};
use baza_storage::crypto::age::AgeKey;

mod manifest;
mod restore;

pub use restore::{RestoreCheckReport, RestoreOptions};

use crate::BazaManager;

use self::manifest::{BackupManifestWrite, write_backup_manifest};

impl BazaManager {
    /// Creates an encrypted backup generation in an existing absolute directory.
    ///
    /// Requires unlocked storage with no staged changes. The generation contains copied key, DB,
    /// and blob artifacts plus an authenticated manifest; it does not provide an atomic snapshot
    /// when live storage changes concurrently.
    pub fn backup(&self, backup_dir: &str) -> Result<()> {
        log::debug!("backup_dir: {backup_dir}");

        ensure!(
            Path::new(backup_dir).is_absolute(),
            "backup dir path must be absolute"
        );

        ensure_dir_exists(backup_dir)?;

        let storage_key = AgeKey::from_age_x25519_key(self.get_unlocked_storage_key()?)?;
        let baza = self.open()?;

        if baza.has_staged_documents() {
            bail!("Can't backup: there are staged changes");
        }

        let data_dir = format!("{backup_dir}/data");
        create_dir_if_not_exist(&data_dir)?;

        let now = Timestamp::now().format_time("[year]-[month padding:zero]-[day padding:zero]_[hour padding:zero]-[minute padding:zero]-[second padding:zero]").expect("must be valid format");

        // copy key file as [timestamp].key.age
        let backup_key_file = format!("{backup_dir}/{now}.{}", self.paths.key_file_name);
        // copy storage file as [timestamp].baza.gz.age
        let backup_storage_file = format!(
            "{backup_dir}/{now}.{}",
            self.paths.storage_main_db_file_name
        );

        let mut fs_tx = FsTransaction::new();

        fs_tx.copy_file_new(&self.paths.key_file, &backup_key_file)?;
        log::info!("Backup: copied key file into {backup_key_file}");

        fs_tx.copy_file_new(&self.paths.storage_main_db_file, &backup_storage_file)?;
        log::info!("Backup: copied main storage file into {backup_storage_file}");

        let mut blob_ids = baza
            .referenced_storage_blob_ids()
            .into_iter()
            .collect::<Vec<_>>();
        blob_ids.sort_by_key(ToString::to_string);

        let blobs = list_files(&self.paths.storage_data_dir)?;
        log::info!("Backup: found {} BLOBs", blobs.len());

        // copy blobs if needed
        let mut blob_count = 0;
        for blob_file_path in blobs {
            let blob_file_name = get_file_name(&blob_file_path);

            let backup_blob_path = format!("{data_dir}/{blob_file_name}");

            // check if backup file exists
            if !file_exists(&backup_blob_path)? {
                // copy blob
                fs_tx.copy_file_new(&blob_file_path, &backup_blob_path)?;
                log::debug!("Created blob backup {backup_blob_path}");

                blob_count += 1;
            }
        }

        if blob_count > 0 {
            log::info!("Back up: copied {blob_count} new blobs");
        } else {
            log::info!("Back up: no new blobs to backup");
        }

        drop(baza);

        let manifest_path = write_backup_manifest(
            &mut fs_tx,
            BackupManifestWrite {
                backup_dir,
                timestamp: &now,
                key_path: &backup_key_file,
                db_path: &backup_storage_file,
                storage_key,
                blob_ids,
            },
        )?;
        fs_tx.commit()?;
        log::info!("Backup: wrote manifest {manifest_path}");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use baza_common::TempFile;

    use crate::BazaManager;

    #[test]
    fn backup_rejects_staged_changes() -> Result<()> {
        let temp_dir = TempFile::new_with_details("baza_backup", "");
        temp_dir.mkdir()?;
        let manager = BazaManager::new_for_tests(&temp_dir.path);

        let file = temp_dir.new_child("asset-one");
        file.write_str("asset content")?;
        {
            let mut baza = manager.open_mut()?;
            baza.create_asset(&file.path)?;
            baza.save_changes()?;
        }

        let backup_dir = format!("{}/backup", temp_dir.path);
        std::fs::create_dir(&backup_dir)?;

        let err = manager
            .backup(&backup_dir)
            .expect_err("backup must reject staged changes");
        assert!(err.to_string().contains("staged changes"));

        Ok(())
    }
}
