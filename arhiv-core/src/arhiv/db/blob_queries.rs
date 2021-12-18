use anyhow::*;

use rs_utils::{is_same_filesystem, log, FsTransaction};

use crate::entities::{BLOBId, BLOB};

pub trait BLOBQueries {
    fn get_data_dir(&self) -> &str;

    fn get_blob(&self, blob_id: &BLOBId) -> BLOB {
        BLOB::new(blob_id.clone(), self.get_data_dir())
    }
}

pub trait MutableBLOBQueries: BLOBQueries {
    fn get_fs_tx(&mut self) -> &mut FsTransaction;

    fn add_blob(&mut self, file_path: &str, move_file: bool) -> Result<BLOBId> {
        let blob_id = BLOBId::from_file(file_path)?;

        let blob = self.get_blob(&blob_id);

        if blob.exists()? {
            log::debug!("blob {} already exists", blob_id);

            return Ok(blob_id);
        }

        let data_dir = self.get_data_dir().to_string();
        let fs_tx = self.get_fs_tx();

        if move_file {
            fs_tx.move_file(file_path, blob.file_path)?;
            log::debug!("Moved new blob {} from {}", blob_id, file_path);
        } else if is_same_filesystem(file_path, &data_dir)? {
            fs_tx.hard_link_file(file_path, blob.file_path)?;
            log::debug!("Hard linked new blob {} from {}", blob_id, file_path);
        } else {
            fs_tx.copy_file(file_path, blob.file_path)?;
            log::debug!("Copied new blob {} from {}", blob_id, file_path);
        }

        log::info!("Created blob {} from {}", &blob_id, file_path);

        Ok(blob_id)
    }

    fn remove_blob(&mut self, blob_id: &BLOBId) -> Result<()> {
        let blob = self.get_blob(blob_id);

        self.get_fs_tx().remove_file(&blob.file_path)?;

        log::debug!("Removed blob {} from {}", blob_id, blob.file_path);

        Ok(())
    }

    // FIXME pub fn get_blob_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_blob_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}
