use anyhow::*;
use rs_utils::file_exists;

pub struct AttachmentData {
    pub hash: String,
    pub path: String,
}

impl AttachmentData {
    pub fn new(hash: String, path: String) -> Self {
        AttachmentData { hash, path }
    }

    pub fn exists(&self) -> Result<bool> {
        file_exists(&self.path)
    }
}
