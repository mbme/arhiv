use anyhow::*;
use rs_utils::{file_exists, get_file_hash_sha256};

use crate::entities::Id;

pub struct AttachmentData {
    pub id: Id,
    pub path: String,
}

impl AttachmentData {
    pub fn new(id: Id, path: String) -> Self {
        AttachmentData { id, path }
    }

    pub fn exists(&self) -> Result<bool> {
        file_exists(&self.path)
    }

    pub fn get_hash(&self) -> Result<String> {
        get_file_hash_sha256(&self.path)
    }
}
