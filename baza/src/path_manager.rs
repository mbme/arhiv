use std::fs;
use std::path::Path;

use anyhow::{ensure, Result};

use rs_utils::{ensure_dir_exists, ensure_file_exists, TempFile};

use crate::entities::BLOBId;

#[derive(Debug)]
pub struct PathManager {
    pub root_dir: String,
    pub data_dir: String,
    pub downloads_dir: String,
    pub db_file: String,
    pub lock_file: String,

    pub db2_file: String,
    pub db2_data_dir: String,
    pub state_file: String,
    pub state_data_dir: String,
}

impl PathManager {
    pub fn new(root_dir: String) -> PathManager {
        let data_dir = format!("{}/data", &root_dir);
        let downloads_dir = format!("{}/downloads", &root_dir);
        let db_file = format!("{}/arhiv.sqlite", &root_dir);
        let lock_file = format!("{}/arhiv.lock", &root_dir);

        // FIXME
        let db2_file = db_file.clone(); // .gz.c1
        let db2_data_dir = data_dir.clone();
        let state_file = format!("{}/state", &root_dir);
        let state_data_dir = format!("{}/state-data", &root_dir);

        PathManager {
            db2_file,
            db2_data_dir,
            state_file,
            state_data_dir,

            root_dir,
            data_dir,
            downloads_dir,
            db_file,
            lock_file,
        }
    }

    pub fn assert_dirs_exist(&self) -> Result<()> {
        ensure_dir_exists(&self.root_dir)?;
        ensure_dir_exists(&self.data_dir)?;
        ensure_dir_exists(&self.downloads_dir)?;

        Ok(())
    }

    pub fn assert_db_file_exists(&self) -> Result<()> {
        ensure_file_exists(&self.db_file)?;

        Ok(())
    }

    pub fn create_dirs(&self) -> Result<()> {
        let path = Path::new(&self.root_dir);

        ensure!(
            path.is_absolute(),
            "path must be absolute: {}",
            self.root_dir
        );

        ensure!(!path.exists(), "path already exists: {}", self.root_dir);

        fs::create_dir(&self.root_dir)?;
        fs::create_dir(&self.data_dir)?;
        fs::create_dir(&self.downloads_dir)?;

        Ok(())
    }

    pub fn new_temp_file(&self, prefix: &str) -> TempFile {
        TempFile::new_in_dir(&self.downloads_dir, prefix)
    }

    pub fn get_db2_blob_path(&self, id: &BLOBId) -> String {
        format!("{}/{id}", self.db2_data_dir)
    }

    pub fn get_state_blob_path(&self, id: &BLOBId) -> String {
        format!("{}/{id}", self.state_data_dir)
    }
}
