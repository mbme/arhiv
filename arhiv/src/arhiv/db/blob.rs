use anyhow::*;

use rs_utils::{ensure_file_exists, log, FsTransaction};

use super::AttachmentData;
use crate::entities::BLOBHash;

pub trait BLOBQueries {
    fn get_data_dir(&self) -> &str;

    fn get_attachment_data_path(&self, hash: &BLOBHash) -> String {
        format!("{}/{}", self.get_data_dir(), hash)
    }

    fn get_attachment_data(&self, hash: BLOBHash) -> AttachmentData {
        let path = self.get_attachment_data_path(&hash);

        AttachmentData::new(hash, path)
    }
}

pub trait MutableBLOBQueries: BLOBQueries {
    fn get_fs_tx(&mut self) -> &mut FsTransaction;

    fn add_attachment_data(&mut self, file_path: &str, copy: bool) -> Result<BLOBHash> {
        ensure_file_exists(&file_path)?;

        let hash = BLOBHash::from_file(file_path)?;

        let attachment_data = self.get_attachment_data(hash.clone());

        // blob already exists, ignoring
        if attachment_data.exists()? {
            log::warn!(
                "attachment data for {} already exists: {}",
                file_path,
                &hash
            );
            return Ok(hash);
        }

        let fs_tx = self.get_fs_tx();
        if copy {
            fs_tx.copy_file(file_path.to_string(), attachment_data.path)?;
        } else {
            fs_tx.hard_link_file(file_path.to_string(), attachment_data.path)?;
        }

        log::debug!(
            "{} new attachment data {} from {}",
            if copy { "Copied" } else { "Hard linked" },
            &hash,
            file_path
        );

        Ok(hash)
    }

    fn remove_attachment_data(&mut self, hash: &BLOBHash) {
        let attachment_data_path = self.get_attachment_data_path(hash);

        self.get_fs_tx().remove_file(attachment_data_path);
    }

    // FIXME pub fn get_attachment_data_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_attachment_data_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}
