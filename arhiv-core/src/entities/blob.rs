use anyhow::*;

use rs_utils::file_exists;

use crate::entities::BLOBId;

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
}
