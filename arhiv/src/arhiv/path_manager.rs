use anyhow::*;
use rs_utils::{ensure_dir_exists, ensure_file_exists};
use std::fs;
use std::path::Path;

pub struct PathManager {
    root_dir: String,
    pub data_dir: String,
    pub db_file: String,
}

impl PathManager {
    pub fn new<S: Into<String>>(root_dir: S) -> PathManager {
        let root_dir = root_dir.into();
        let data_dir = format!("{}/data", &root_dir);
        let db_file = format!("{}/arhiv.sqlite", &root_dir);

        PathManager {
            root_dir,
            data_dir,
            db_file,
        }
    }

    pub fn assert_dirs_exist(&self) -> Result<()> {
        ensure_dir_exists(&self.root_dir)?;
        ensure_dir_exists(&self.data_dir)?;

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

        Ok(())
    }
}
