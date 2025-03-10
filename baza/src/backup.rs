use std::{fs, path::Path};

use anyhow::{ensure, Result};

use rs_utils::{
    create_dir_if_not_exist, ensure_dir_exists, file_exists, format_time, get_file_name,
    list_files, log, now,
};

use crate::baza2::BazaManager;

impl BazaManager {
    pub fn backup(&self, backup_dir: &str) -> Result<()> {
        log::debug!("backup_dir: {backup_dir}");

        ensure!(
            Path::new(backup_dir).is_absolute(),
            "backup dir path must be absolute"
        );

        ensure_dir_exists(backup_dir)?;

        let baza = self.open()?;

        // warn if there are uncommitted changes
        if baza.has_staged_documents() {
            log::warn!("Backup: there are uncommitted changes, they won't be backed up");
        }

        let data_dir = format!("{backup_dir}/data");
        create_dir_if_not_exist(&data_dir)?;

        let now = format_time(now(), "%Y-%m-%d_%H-%M-%S");

        // copy key file as [timestamp].key.age
        let backup_key_file = format!("{backup_dir}/{now}.{}", self.paths.key_file_name);
        fs::copy(&self.paths.key_file, &backup_key_file)?;
        log::info!("Backup: copied key file into {backup_key_file}");

        // copy storage file as [timestamp].baza.gz.age
        let backup_storage_file = format!(
            "{backup_dir}/{now}.{}",
            self.paths.storage_main_db_file_name
        );
        fs::copy(&self.paths.storage_main_db_file, &backup_storage_file)?;
        log::info!("Backup: copied main storage file into {backup_storage_file}");

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
                fs::copy(&blob_file_path, &backup_blob_path)?;
                log::debug!("Created blob backup {backup_blob_path}");

                blob_count += 1;
            }
        }

        if blob_count > 0 {
            log::info!("Back up: copied {blob_count} new blobs");
        } else {
            log::info!("Back up: no new blobs to backup");
        }

        Ok(())
    }
}
