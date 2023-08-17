use anyhow::Result;

use rs_utils::{file_exists, get_file_size, get_media_type};

use crate::entities::BLOBId;

#[derive(Hash, Eq, PartialEq)]
pub struct BLOB {
    pub id: BLOBId,
    pub file_path: String,
}

impl BLOB {
    #[must_use]
    pub fn new(id: BLOBId, data_dir: &str) -> Self {
        BLOB {
            file_path: format!("{}/{}", data_dir, id.get_file_name()),
            id,
        }
    }

    pub fn exists(&self) -> Result<bool> {
        file_exists(&self.file_path)
    }

    pub fn get_size(&self) -> Result<u64> {
        get_file_size(&self.file_path)
    }

    pub fn get_media_type(&self) -> Result<String> {
        get_media_type(&self.file_path)
    }
}
