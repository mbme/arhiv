use anyhow::*;

use rs_utils::{ensure_file_exists, is_same_filesystem, log, FsTransaction};

use super::AttachmentData;
use crate::entities::Id;

pub trait BLOBQueries {
    fn get_data_dir(&self) -> &str;

    fn get_attachment_data(&self, id: &Id) -> AttachmentData {
        AttachmentData::new(id.clone(), format!("{}/{}", self.get_data_dir(), id))
    }
}

pub trait MutableBLOBQueries: BLOBQueries {
    fn get_fs_tx(&mut self) -> &mut FsTransaction;

    fn add_attachment_data(&mut self, id: &Id, file_path: &str, move_file: bool) -> Result<()> {
        ensure_file_exists(file_path)?;

        let attachment_data = self.get_attachment_data(id);

        ensure!(
            !attachment_data.exists()?,
            "attachment data {} already exists",
            id
        );

        let data_dir = self.get_data_dir().to_string();
        let fs_tx = self.get_fs_tx();

        if move_file {
            fs_tx.move_file(file_path, attachment_data.path)?;
            log::debug!("Moved new attachment data {} from {}", id, file_path);
        } else if is_same_filesystem(file_path, &data_dir)? {
            fs_tx.hard_link_file(file_path, attachment_data.path)?;
            log::debug!("Hard linked new attachment data {} from {}", id, file_path);
        } else {
            fs_tx.copy_file(file_path, attachment_data.path)?;
            log::debug!("Copied new attachment data {} from {}", id, file_path);
        }

        Ok(())
    }

    fn remove_attachment_data(&mut self, id: &Id) -> Result<()> {
        let attachment_data = self.get_attachment_data(id);

        self.get_fs_tx().remove_file(&attachment_data.path)?;

        log::debug!(
            "Removed attachment data {} from {}",
            id,
            attachment_data.path
        );

        Ok(())
    }

    // FIXME pub fn get_attachment_data_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_attachment_data_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}
