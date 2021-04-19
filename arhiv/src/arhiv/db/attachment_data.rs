use crate::entities::Hash;
use anyhow::*;
use rs_utils::file_exists;

pub struct AttachmentData {
    pub hash: Hash,
    pub path: String,
}

impl AttachmentData {
    pub fn new(hash: Hash, path: String) -> Self {
        AttachmentData { hash, path }
    }

    pub fn exists(&self) -> Result<bool> {
        file_exists(&self.path)
    }
}
