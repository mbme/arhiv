use super::db::*;
use super::Arhiv;
use crate::entities::Hash;

use anyhow::*;
use rs_utils::{ensure_dir_exists, file_exists, log, run_command};
use std::{fs, path::Path};

impl Arhiv {
    pub fn backup<S: Into<String>>(&self, backup_dir: S) -> Result<()> {
        let backup = BackupPaths::new(backup_dir.into());
        backup.check()?;

        // 1. vacuum the db so that WAL is written into db
        self.db.cleanup()?;

        // 2. copy & archive db file
        run_command(
            "zstd",
            vec![&self.db.get_db_file(), "-o", &backup.backup_db_file],
        )?;
        log::info!("Created arhiv backup {}", &backup.backup_db_file);

        // 3. copy all data files if needed
        let mut blob_count = 0;
        let conn = self.db.get_connection()?;
        for entry in self.db.iter_blobs()? {
            let hash = entry?;

            // check if backup file exists
            if backup.blob_exists(&hash)? {
                log::trace!("Blob {} backup already exists, skipping", &hash);
                continue;
            }

            // check if blob is in use
            if !conn.is_blob_in_use(&hash)? {
                log::warn!("Blob {} is unused, skipping", &hash);
                continue;
            }

            // copy blob
            fs::copy(
                &conn.get_attachment_data(hash.clone()).path,
                backup.get_blob_path(&hash),
            )?;
            log::debug!("Created blob {} backup", &hash);

            blob_count += 1;
        }

        log::info!("Backed up {} new blobs", blob_count);

        Ok(())
    }
}

struct BackupPaths {
    pub backup_dir: String,
    pub data_dir: String,
    pub backup_db_file: String,
}

impl BackupPaths {
    pub fn new(backup_dir: String) -> Self {
        let data_dir = format!("{}/data", &backup_dir);

        let now = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S");
        let backup_db_file = format!("{}/arhiv_{}.sqlite.zst", &backup_dir, now);

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

    pub fn get_blob_path(&self, blob_hash: &Hash) -> String {
        format!("{}/{}", &self.data_dir, blob_hash)
    }

    pub fn blob_exists(&self, blob_hash: &Hash) -> Result<bool> {
        file_exists(&self.get_blob_path(blob_hash))
    }
}
