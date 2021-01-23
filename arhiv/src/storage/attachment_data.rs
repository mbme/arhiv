use super::path_manager::PathManager;
use super::Id;
use anyhow::*;
use rs_utils::{file_exists, get_file_hash_sha256};

pub struct AttachmentData<'a> {
    pub id: Id,
    path_manager: &'a PathManager,
}

impl<'a> AttachmentData<'a> {
    pub fn new(id: Id, path_manager: &'a PathManager) -> AttachmentData<'a> {
        AttachmentData { id, path_manager }
    }

    pub fn get_committed_file_path(&self) -> String {
        self.path_manager.get_committed_file_path(&self.id)
    }

    pub fn get_staged_file_path(&self) -> String {
        self.path_manager.get_staged_file_path(&self.id)
    }

    pub fn committed_file_exists(&self) -> Result<bool> {
        file_exists(&self.get_committed_file_path())
    }

    pub fn staged_file_exists(&self) -> Result<bool> {
        file_exists(&self.get_staged_file_path())
    }

    pub fn get_staged_file_hash(&self) -> Result<String> {
        get_file_hash_sha256(&self.get_staged_file_path())
    }
}
