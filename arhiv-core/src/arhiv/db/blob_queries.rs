use std::{collections::HashSet, fs};

use anyhow::{anyhow, ensure, Context, Result};

use rs_utils::{file_exists, is_same_filesystem, log};

use crate::{
    entities::{BLOBId, BLOB},
    ArhivConnection,
};

impl ArhivConnection {
    pub(crate) fn get_blob(&self, blob_id: &BLOBId) -> BLOB {
        BLOB::new(blob_id.clone(), self.get_data_dir())
    }

    pub(crate) fn list_local_blobs(&self) -> Result<HashSet<BLOBId>> {
        let items = fs::read_dir(self.get_data_dir())?
            .map(|item| {
                let entry = item.context("Failed to read data entry")?;

                let entry_path = entry.path();

                ensure!(
                    entry_path.is_file(),
                    "{} isn't a file",
                    entry_path.to_string_lossy()
                );

                entry_path
                    .file_name()
                    .ok_or_else(|| anyhow!("Failed to read file name"))
                    .map(|value| value.to_string_lossy().to_string())
                    .and_then(|value| BLOBId::from_file_name(&value))
            })
            .collect::<Result<HashSet<_>>>()?;

        Ok(items)
    }

    pub(crate) fn add_blob(&mut self, file_path: &str, move_file: bool) -> Result<BLOBId> {
        ensure!(
            file_exists(file_path)?,
            "BLOB source must exist and must be a file"
        );

        let blob_id = BLOBId::from_file(file_path)?;

        let blob = self.get_blob(&blob_id);

        if blob.exists()? {
            log::debug!("blob {} already exists", blob_id);

            return Ok(blob_id);
        }

        let data_dir = self.get_data_dir().to_string();
        let fs_tx = self.get_fs_tx()?;

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

    pub(crate) fn remove_blob(&mut self, blob_id: &BLOBId) -> Result<()> {
        let blob = self.get_blob(blob_id);

        self.get_fs_tx()?.remove_file(&blob.file_path)?;

        log::debug!("Removed blob {} from {}", blob_id, blob.file_path);

        Ok(())
    }

    // FIXME pub fn get_blob_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_blob_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}
