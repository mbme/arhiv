use std::{collections::HashSet, fmt::Display};

use anyhow::Result;

use rs_utils::{
    create_dir_if_not_exist, dir_exists, file_exists, get_file_modification_time, get_file_name,
    list_files, Timestamp,
};

use crate::entities::BLOBId;

const BLOB_EXT: &str = ".age";

const STORAGE_EXT: &str = ".gz.age";

const STATE_EXT: &str = ".gz.age";

#[derive(Clone)]
pub struct BazaPaths {
    pub key_file_name: String,
    pub key_file: String,

    pub storage_dir: String,
    pub storage_main_db_file_name: String,
    pub storage_main_db_file: String,
    pub storage_data_dir: String,

    pub state_dir: String,
    pub state_file: String,
    pub state_data_dir: String,

    pub lock_file: String,
}

impl BazaPaths {
    pub fn new(storage_dir: String, state_dir: String) -> Self {
        let key_file_name = "key.age".to_string();
        let key_file = format!("{storage_dir}/{key_file_name}");

        let storage_main_db_file_name = format!("baza{STORAGE_EXT}");
        let storage_main_db_file = format!("{storage_dir}/{storage_main_db_file_name}");
        let storage_data_dir = format!("{storage_dir}/data");

        let state_file = format!("{state_dir}/state{STATE_EXT}");
        let state_data_dir = format!("{state_dir}/data");

        let lock_file = format!("{state_dir}/baza.lock");

        Self {
            key_file_name,
            key_file,

            storage_dir,
            storage_main_db_file_name,
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
        list_blobs_in_dir(&self.storage_data_dir, BLOB_EXT)
    }

    pub fn list_state_blobs(&self) -> Result<HashSet<BLOBId>> {
        list_blobs_in_dir(&self.state_data_dir, BLOB_EXT)
    }

    pub fn list_blobs(&self) -> Result<HashSet<BLOBId>> {
        let mut ids = self.list_storage_blobs()?;
        let local_ids = self.list_state_blobs()?;

        ids.extend(local_ids);

        Ok(ids)
    }

    pub fn storage_dir_exists(&self) -> Result<bool> {
        dir_exists(&self.storage_dir)
    }

    pub fn storage_blob_exists(&self, id: &BLOBId) -> Result<bool> {
        let file = self.get_storage_blob_path(id);

        file_exists(&file)
    }

    pub fn key_file_exists(&self) -> Result<bool> {
        file_exists(&self.key_file)
    }

    pub fn state_file_exists(&self) -> Result<bool> {
        file_exists(&self.state_file)
    }

    pub fn read_state_file_modification_time(&self) -> Result<Timestamp> {
        get_file_modification_time(&self.state_file)
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

fn list_blobs_in_dir(dir: &str, trim_ext: &str) -> Result<HashSet<BLOBId>> {
    let files = list_files(dir)?;

    let ids = files
        .into_iter()
        .filter_map(|file_path| {
            let file_name = get_file_name(file_path.trim_end_matches(trim_ext));

            BLOBId::from_string(file_name).ok()
        })
        .collect();

    Ok(ids)
}
