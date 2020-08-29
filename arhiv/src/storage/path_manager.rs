use crate::utils::{ensure_dir_exists, ensure_file_exists};
use anyhow::*;
use std::fs;
use std::path::Path;

pub struct PathManager {
    root_path: String,
}

impl PathManager {
    pub fn new<S: Into<String>>(root_path: S) -> PathManager {
        PathManager {
            root_path: root_path.into(),
        }
    }

    pub fn get_data_directory(&self) -> String {
        format!("{}/data", self.root_path)
    }

    pub fn get_temp_data_directory(&self) -> String {
        format!("{}/temp-data", self.root_path)
    }

    pub fn get_db_file(&self) -> String {
        format!("{}/arhiv.sqlite", self.root_path)
    }

    pub fn assert_dirs_exist(&self) -> Result<()> {
        ensure_dir_exists(&self.root_path)?;
        ensure_dir_exists(&self.get_data_directory())?;
        ensure_dir_exists(&self.get_temp_data_directory())?;

        Ok(())
    }

    pub fn assert_db_file_exists(&self) -> Result<()> {
        ensure_file_exists(&self.get_db_file())?;

        Ok(())
    }

    pub fn create_dirs(&self) -> Result<()> {
        let path = Path::new(&self.root_path);

        if !path.is_absolute() {
            bail!("path must be absolute: {}", &self.root_path);
        }

        if path.exists() {
            bail!("path already exists: {}", &self.root_path);
        }

        fs::create_dir(&self.root_path)?;
        fs::create_dir(&self.get_data_directory())?;
        fs::create_dir(&self.get_temp_data_directory())?;

        Ok(())
    }
}
