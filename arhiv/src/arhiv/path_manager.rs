use anyhow::*;
use rs_utils::{ensure_dir_exists, ensure_file_exists, log};
use std::fs::{self, ReadDir};
use std::path::Path;

use crate::entities::Hash;

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

    pub fn iter_blobs(&self) -> Result<DataBlob> {
        let read_dir = fs::read_dir(&self.data_dir)?;

        Ok(DataBlob(read_dir))
    }
}

pub struct DataBlob(ReadDir);

impl Iterator for DataBlob {
    type Item = Result<Hash>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = match self.0.next() {
                Some(Ok(entry)) => entry,
                Some(Err(err)) => return Some(Err(err).context("Failed to read data entry")),
                None => return None,
            };

            let entry_path = entry.path();
            if entry_path.is_file() {
                let file_name = entry_path
                    .file_name()
                    .ok_or(anyhow!("Failed to read file name"))
                    .map(|value| Hash::from_string(value.to_string_lossy().to_string()));

                return Some(file_name);
            } else {
                log::warn!("Blob {} isn't a file", entry_path.to_string_lossy());
            }
        }
    }
}
