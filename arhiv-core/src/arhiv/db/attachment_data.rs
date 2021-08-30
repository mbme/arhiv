use anyhow::*;

use rs_utils::file_exists;

use crate::entities::Id;

pub struct AttachmentData {
    pub id: Id,
    pub path: String,
}

impl AttachmentData {
    #[must_use]
    pub fn new(id: Id, path: String) -> Self {
        AttachmentData { id, path }
    }

    pub fn exists(&self) -> Result<bool> {
        file_exists(&self.path)
    }
}
