use anyhow::*;
use rs_utils::{ensure_dir_exists, ensure_file_exists};
use std::fs;
use std::path::Path;

pub struct PathManager {
    root_dir: String,
}

impl PathManager {
    pub fn new<S: Into<String>>(root_dir: S) -> PathManager {
        PathManager {
            root_dir: root_dir.into(),
        }
    }

    pub fn get_data_directory(&self) -> String {
        format!("{}/data", self.root_dir)
    }

    pub fn get_db_file(&self) -> String {
        format!("{}/arhiv.sqlite", self.root_dir)
    }

    pub fn get_attachment_data_path(&self, filename: &str) -> String {
        format!("{}/{}", self.get_data_directory(), filename)
    }

    pub fn assert_dirs_exist(&self) -> Result<()> {
        ensure_dir_exists(&self.root_dir)?;
        ensure_dir_exists(&self.get_data_directory())?;

        Ok(())
    }

    pub fn assert_db_file_exists(&self) -> Result<()> {
        ensure_file_exists(&self.get_db_file())?;

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
        fs::create_dir(&self.get_data_directory())?;

        Ok(())
    }
}
