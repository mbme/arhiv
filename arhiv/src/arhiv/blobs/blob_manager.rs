use anyhow::*;
use rs_utils::{ensure_file_exists, log::debug, FsTransaction};

use super::AttachmentData;
use crate::entities::Hash;

pub struct BlobManager {
    data_dir: String,
}

impl BlobManager {
    pub fn new<S: Into<String>>(data_dir: S) -> Self {
        BlobManager {
            data_dir: data_dir.into(),
        }
    }

    fn get_attachment_data_path(&self, hash: &Hash) -> String {
        format!("{}/{}", &self.data_dir, hash)
    }

    pub fn get_attachment_data(&self, hash: Hash) -> AttachmentData {
        let path = self.get_attachment_data_path(&hash);

        AttachmentData::new(hash, path)
    }

    pub fn add_attachment_data(
        &self,
        fs_tx: &mut FsTransaction,
        file_path: &str,
        copy: bool,
    ) -> Result<Hash> {
        ensure_file_exists(&file_path)?;

        let hash = Hash::from_file(file_path)?;

        let attachment_data_path = self.get_attachment_data_path(&hash);

        if copy {
            fs_tx.copy_file(file_path.to_string(), attachment_data_path)?;
        } else {
            fs_tx.hard_link_file(file_path.to_string(), attachment_data_path)?;
        }

        debug!(
            "{} new attachment data {} from {}",
            if copy { "Copied" } else { "Hard linked" },
            &hash,
            file_path
        );

        Ok(hash)
    }

    pub fn remove_attachment_data(&self, fs_tx: &mut FsTransaction, hash: &Hash) {
        let attachment_data_path = self.get_attachment_data_path(hash);

        fs_tx.remove_file(attachment_data_path);
    }

    // FIXME pub fn get_attachment_data_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_attachment_data_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}
