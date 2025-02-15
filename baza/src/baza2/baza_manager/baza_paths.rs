use std::{collections::HashSet, fmt::Display};

use anyhow::Result;

use rs_utils::{create_dir_if_not_exist, file_exists, list_files};

use crate::{entities::BLOBId, get_local_blob_ids};

const BLOB_EXT: &str = ".c1";

const STORAGE_EXT: &str = ".gz.c1";

const STATE_EXT: &str = ".c1";

pub struct BazaPaths {
    pub storage_dir: String,
    pub storage_main_db_file: String,
    pub storage_data_dir: String,

    pub state_dir: String,
    pub state_file: String,
    pub state_data_dir: String,

    pub lock_file: String,
}

impl BazaPaths {
    pub fn new(storage_dir: String, state_dir: String) -> Self {
        let storage_main_db_file = format!("{storage_dir}/baza{STORAGE_EXT}");
        let storage_data_dir = format!("{storage_dir}/data");

        let state_file = format!("{state_dir}/state{STATE_EXT}");
        let state_data_dir = format!("{state_dir}/data");

        let lock_file = format!("{state_dir}/baza.lock");

        Self {
            storage_dir,
            storage_main_db_file,
            storage_data_dir,

            state_dir,
            state_file,
            state_data_dir,

            lock_file,
        }
    }

    pub fn ensure_dirs_exist(&self) -> Result<()> {
        create_dir_if_not_exist(&self.storage_dir)?;
        create_dir_if_not_exist(&self.storage_data_dir)?;

        create_dir_if_not_exist(&self.state_dir)?;
        create_dir_if_not_exist(&self.state_data_dir)?;

        Ok(())
    }

    pub fn list_storage_db_files(&self) -> Result<Vec<String>> {
        let result = list_files(&self.storage_dir)?
            .into_iter()
            .filter(|file| file.ends_with(STORAGE_EXT))
            .collect();

        Ok(result)
    }

    #[cfg(test)]
    pub fn get_storage_file(&self, storage_name: &str) -> String {
        format!("{}/{storage_name}{STORAGE_EXT}", self.storage_dir)
    }

    pub fn get_storage_blob_path(&self, id: &BLOBId) -> String {
        format!("{}/{id}{BLOB_EXT}", self.storage_data_dir)
    }

    pub fn get_state_blob_path(&self, id: &BLOBId) -> String {
        format!("{}/{id}{BLOB_EXT}", self.state_data_dir)
    }

    pub fn list_storage_blobs(&self) -> Result<HashSet<BLOBId>> {
        get_local_blob_ids(&self.storage_data_dir, BLOB_EXT)
    }

    pub fn list_state_blobs(&self) -> Result<HashSet<BLOBId>> {
        get_local_blob_ids(&self.state_data_dir, BLOB_EXT)
    }

    pub fn list_blobs(&self) -> Result<HashSet<BLOBId>> {
        let mut ids = self.list_storage_blobs()?;
        let local_ids = self.list_state_blobs()?;

        ids.extend(local_ids);

        Ok(ids)
    }

    pub fn storage_blob_exists(&self, id: &BLOBId) -> Result<bool> {
        let file = self.get_storage_blob_path(id);

        file_exists(&file)
    }
}

impl Display for BazaPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[BazaPaths storage: {}  state: {}]",
            self.storage_dir, self.state_dir
        )
    }
}
