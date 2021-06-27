use anyhow::*;

use rs_utils::{ensure_file_exists, log, FsTransaction};

use super::AttachmentData;
use crate::entities::Id;

pub trait BLOBQueries {
    fn get_data_dir(&self) -> &str;

    fn get_attachment_data_path(&self, id: &Id) -> String {
        format!("{}/{}", self.get_data_dir(), id)
    }

    fn get_attachment_data(&self, id: &Id) -> AttachmentData {
        let path = self.get_attachment_data_path(&id);

        AttachmentData::new(id.clone(), path)
    }
}

pub trait MutableBLOBQueries: BLOBQueries {
    fn get_fs_tx(&mut self) -> &mut FsTransaction;

    fn add_attachment_data(&mut self, id: &Id, file_path: &str, copy: bool) -> Result<()> {
        ensure_file_exists(&file_path)?;

        let attachment_data = self.get_attachment_data(id);

        ensure!(
            !attachment_data.exists()?,
            "attachment data {} already exists",
            id
        );

        let fs_tx = self.get_fs_tx();
        if copy {
            fs_tx.copy_file(file_path.to_string(), attachment_data.path)?;
        } else {
            fs_tx.hard_link_file(file_path.to_string(), attachment_data.path)?;
        }

        log::debug!(
            "{} new attachment data {} from {}",
            if copy { "Copied" } else { "Hard linked" },
            id,
            file_path
        );

        Ok(())
    }

    fn remove_attachment_data(&mut self, id: &Id) -> Result<()> {
        let attachment_data_path = self.get_attachment_data_path(id);

        self.get_fs_tx().remove_file(&attachment_data_path)?;

        log::debug!(
            "Removed attachment data {} from {}",
            id,
            attachment_data_path
        );

        Ok(())
    }

    // FIXME pub fn get_attachment_data_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_attachment_data_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}
