use crate::entities::Id;

use super::path_manager::PathManager;
use anyhow::*;
use rs_utils::{file_exists, get_file_hash_sha256};

pub struct DataService {
    path_manager: PathManager,
}

impl DataService {
    pub fn new(path_manager: PathManager) -> DataService {
        DataService { path_manager }
    }

    pub fn get_committed_file_path(&self, id: &Id) -> String {
        self.path_manager.get_committed_file_path(id)
    }

    pub fn get_staged_file_path(&self, id: &Id) -> String {
        self.path_manager.get_staged_file_path(id)
    }

    pub fn committed_file_exists(&self, id: &Id) -> Result<bool> {
        file_exists(&self.get_committed_file_path(id))
    }

    pub fn staged_file_exists(&self, id: &Id) -> Result<bool> {
        file_exists(&self.get_staged_file_path(id))
    }

    pub fn get_staged_file_hash(&self, id: &Id) -> Result<String> {
        get_file_hash_sha256(&self.get_staged_file_path(id))
    }
}
