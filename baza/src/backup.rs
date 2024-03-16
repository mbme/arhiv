use std::{fs, path::Path};

use anyhow::{bail, ensure, Result};

use rs_utils::{ensure_dir_exists, file_exists, format_time, into_absolute_path, log, now, ZStd};

use crate::{entities::BLOBId, Baza};

impl Baza {
    pub fn backup(&self, backup_dir: &str) -> Result<()> {
        let zstd = ZStd::check()?;

        ensure!(!backup_dir.is_empty(), "coudn't determine backup dir");

        let backup_dir = into_absolute_path(backup_dir, true)?;

        log::debug!("backup_dir: {}", &backup_dir);

        let app_name = self.get_schema().get_app_name();

        let backup = BackupPaths::new(app_name, backup_dir);
        backup.check()?;

        // 1. cleanup the db
        self.get_db().vacuum()?;

        // 2. copy & compress db file
        zstd.compress(&self.get_path_manager().db_file, &backup.backup_db_file)?;
        log::info!("Created {app_name} backup: {}", &backup.backup_db_file);

        // 3. copy all data files if needed
        let mut blob_count = 0;
        let conn = self.get_connection()?;
        for blob_id in conn.get_local_blob_ids()? {
            // check if backup file exists
            if backup.blob_exists(&blob_id)? {
                log::trace!("Blob {} backup already exists, skipping", &blob_id);
                continue;
            }

            // copy blob
            fs::copy(
                &conn.get_blob(&blob_id).file_path,
                backup.get_blob_path(&blob_id),
            )?;
            log::debug!("Created blob {} backup", &blob_id);

            blob_count += 1;
        }

        if blob_count > 0 {
            log::info!("Backed up {} new blobs", blob_count);
        } else {
            log::info!("No new blobs to backup");
        }

        Ok(())
    }
}

struct BackupPaths {
    pub backup_dir: String,
    pub data_dir: String,
    pub backup_db_file: String,
}

impl BackupPaths {
    pub fn new(file_name: &str, backup_dir: String) -> Self {
        let data_dir = format!("{backup_dir}/data");

        let now = format_time(now(), "%Y-%m-%d_%H-%M-%S");
        let backup_db_file = format!("{backup_dir}/{file_name}_{now}.sqlite.zst");

        BackupPaths {
            backup_dir,
            data_dir,
            backup_db_file,
        }
    }

    pub fn check(&self) -> Result<()> {
        ensure!(
            Path::new(&self.backup_dir).is_absolute(),
            "backup dir path must be absolute"
        );

        ensure_dir_exists(&self.backup_dir)?;

        if Path::new(&self.backup_db_file).exists() {
            bail!("Backup {} already exists", &self.backup_db_file);
        }

        // create data dir if needed
        fs::create_dir_all(&self.data_dir)?;

        Ok(())
    }

    pub fn get_blob_path(&self, id: &BLOBId) -> String {
        format!("{}/{}", &self.data_dir, id)
    }

    pub fn blob_exists(&self, id: &BLOBId) -> Result<bool> {
        file_exists(&self.get_blob_path(id))
    }
}
