use anyhow::*;
use rs_utils::{ensure_dir_exists, ensure_file_exists};
use std::path::Path;
use std::{fs, sync::Arc};

use crate::{entities::Id, Config};

pub struct PathManager {
    config: Arc<Config>,
}

impl PathManager {
    pub fn new(config: Arc<Config>) -> PathManager {
        PathManager { config }
    }

    pub fn get_data_directory(&self) -> String {
        format!("{}/data", self.config.arhiv_root)
    }

    pub fn get_temp_data_directory(&self) -> String {
        format!("{}/temp-data", self.config.arhiv_root)
    }

    pub fn get_db_file(&self) -> String {
        format!("{}/arhiv.sqlite", self.config.arhiv_root)
    }

    pub fn get_committed_file_path(&self, id: &Id) -> String {
        format!("{}/{}", self.get_data_directory(), id)
    }

    pub fn get_staged_file_path(&self, id: &Id) -> String {
        format!("{}/{}", self.get_temp_data_directory(), id)
    }

    pub fn get_attachment_data_url(&self, id: &Id) -> Result<String> {
        let prime_url = self
            .config
            .prime_url
            .as_ref()
            .ok_or(anyhow!("config.prime_url is missing"))?;

        Ok(format!("{}/attachment-data/{}", prime_url, id))
    }

    pub fn assert_dirs_exist(&self) -> Result<()> {
        ensure_dir_exists(&self.config.arhiv_root)?;
        ensure_dir_exists(&self.get_data_directory())?;
        ensure_dir_exists(&self.get_temp_data_directory())?;

        Ok(())
    }

    pub fn assert_db_file_exists(&self) -> Result<()> {
        ensure_file_exists(&self.get_db_file())?;

        Ok(())
    }

    pub fn create_dirs(&self) -> Result<()> {
        let path = Path::new(&self.config.arhiv_root);

        ensure!(
            path.is_absolute(),
            "path must be absolute: {}",
            &self.config.arhiv_root
        );

        ensure!(
            !path.exists(),
            "path already exists: {}",
            &self.config.arhiv_root
        );

        fs::create_dir(&self.config.arhiv_root)?;
        fs::create_dir(&self.get_data_directory())?;
        fs::create_dir(&self.get_temp_data_directory())?;

        Ok(())
    }
}
