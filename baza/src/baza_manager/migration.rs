use std::time::Instant;

use anyhow::{Result, ensure};

use rs_utils::{FsTransaction, age::AgeKey, file_exists, log};

use crate::{
    Baza, BazaStorage,
    baza::write_and_encrypt_blob,
    baza_storage::create_container_patch,
    entities::{Document, Id, InstanceId},
};

use super::BazaManager;

impl BazaManager {
    // TODO remove
    pub fn dangerously_create_state(&self, instance_id: InstanceId) -> Result<()> {
        let _lock = self.wait_for_file_lock()?;
        let key = self.acquire_state_read_lock()?.get_key()?.clone();

        Baza::create(instance_id, key, self.paths.clone(), self.schema.clone())?;

        Ok(())
    }

    // TODO remove
    pub fn dangerously_insert_snapshots_into_storage(
        &self,
        new_snapshots: &[Document],
    ) -> Result<()> {
        log::info!(
            "Inserting {} documents into storage db {}",
            new_snapshots.len(),
            self.paths.storage_main_db_file
        );

        let _lock = self.wait_for_file_lock()?;
        let state = self.acquire_state_read_lock()?;

        let mut fs_tx = FsTransaction::new();

        let old_db_file = fs_tx.move_to_backup(self.paths.storage_main_db_file.clone())?;

        let storage = BazaStorage::read_file(&old_db_file, state.get_key()?.clone())?;

        let patch = create_container_patch(new_snapshots.iter())?;
        storage.patch_and_save_to_file(&self.paths.storage_main_db_file, patch)?;

        fs_tx.commit()?;

        Ok(())
    }

    // TODO remove
    pub fn dangerously_insert_blob_into_storage(
        &self,
        file_path: &str,
        asset_id: &Id,
        blob_key: AgeKey,
    ) -> Result<()> {
        log::info!("Adding file {file_path} to storage");

        let _lock = self.wait_for_file_lock()?;

        ensure!(
            file_exists(file_path)?,
            "BLOB source must exist and must be a file"
        );

        let start_time = Instant::now();

        let blob_path = self.paths.get_storage_blob_path(asset_id);
        ensure!(!file_exists(&blob_path)?, "storage BLOB already exists");

        write_and_encrypt_blob(file_path, &blob_path, blob_key)?;

        let duration = start_time.elapsed();
        log::info!("Encrypted file {file_path} in {:?}", duration);

        Ok(())
    }
}
