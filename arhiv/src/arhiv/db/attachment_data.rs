use crate::entities::BLOBHash;
use anyhow::*;
use rs_utils::file_exists;

pub struct AttachmentData {
    pub hash: BLOBHash,
    pub path: String,
}

impl AttachmentData {
    pub fn new(hash: BLOBHash, path: String) -> Self {
        AttachmentData { hash, path }
    }

    pub fn exists(&self) -> Result<bool> {
        file_exists(&self.path)
    }
}
