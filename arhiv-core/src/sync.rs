use anyhow::{ensure, Context, Result};

use baza::sync::changeset_response::ChangesetResponse;
use rs_utils::{log, now};

use crate::{prime_server::PrimeServerRPC, settings::SETTING_LAST_SYNC_TIME};

use super::Arhiv;

impl Arhiv {
    pub async fn sync(&self) -> Result<()> {
        let result = if self.is_prime()? {
            self.sync_locally()
        } else {
            self.sync_remotely().await
        };

        if let Err(ref err) = result {
            log::error!("sync failed: {}", err);

            return result;
        }

        if !self.is_prime()? {
            self.download_missing_blobs().await?;
        }

        log::info!("sync succeeded");

        // update last sync time
        {
            let tx = self.baza.get_tx()?;
            tx.kvs_const_set(SETTING_LAST_SYNC_TIME, &now())?;
            tx.commit()?;
        }

        self.baza.cleanup()?;

        Ok(())
    }

    fn sync_locally(&self) -> Result<()> {
        log::info!("Initiating local sync");

        let mut tx = self.baza.get_tx()?;

        let changeset = tx.generate_changeset()?;
        log::debug!("prepared a changeset {}", changeset);

        tx.delete_local_staged_changes()?;

        self.baza.apply_changeset(&mut tx, changeset)?;

        tx.commit()?;

        log::info!("Completed local sync");

        Ok(())
    }

    async fn sync_remotely(&self) -> Result<()> {
        log::info!("Initiating remote sync");

        let prime_rpc = PrimeServerRPC::new(
            &self.config.prime_url,
            &self.baza.get_path_manager().downloads_dir,
        )?;

        prime_rpc
            .check_connection()
            .await
            .context("remote connection check failed")?;

        let mut tx = self.baza.get_tx()?;

        let changeset = tx.generate_changeset()?;
        let new_blob_ids = tx.get_new_blob_ids()?;
        log::debug!(
            "sync_remotely: starting {}, {} new blobs",
            &changeset,
            new_blob_ids.len()
        );

        let last_update_time = tx.get_last_update_time()?;

        // TODO parallel file upload
        for (index, blob_id) in new_blob_ids.iter().enumerate() {
            let blob = self.baza.get_blob(blob_id)?;

            log::info!("uploading BLOB {} out of {}", index + 1, new_blob_ids.len());
            prime_rpc.upload_blob(&blob).await?;
        }

        if new_blob_ids.is_empty() {
            log::info!("There are no BLOBs to upload");
        } else {
            log::info!("uploaded {} BLOBs", new_blob_ids.len());
        }

        let response: ChangesetResponse = prime_rpc.send_changeset(&changeset).await?;

        log::debug!("sync_remotely: got response {}", &response);

        ensure!(
            last_update_time == self.baza.get_connection()?.get_last_update_time()?,
            "last_update_time must not change",
        );

        tx.delete_local_staged_changes()?;
        tx.apply_changeset_response(response)?;

        tx.commit()?;

        log::info!("Completed remote sync");

        Ok(())
    }

    async fn download_missing_blobs(&self) -> Result<()> {
        let missing_blob_ids = self.baza.get_connection()?.get_missing_blob_ids()?;

        log::debug!("There are {} missing local BLOBs", missing_blob_ids.len());

        if missing_blob_ids.is_empty() {
            log::info!("There are no missing BLOBs to download");
            return Ok(());
        }

        let prime_rpc = PrimeServerRPC::new(
            &self.config.prime_url,
            &self.baza.get_path_manager().downloads_dir,
        )?;

        // TODO parallel file download
        for (index, blob_id) in missing_blob_ids.iter().enumerate() {
            let blob = self.baza.get_blob(blob_id)?;

            log::info!(
                "downloading BLOBS {} of {}",
                index + 1,
                missing_blob_ids.len()
            );
            prime_rpc.download_blob(&blob).await?;
        }

        log::info!("finished downloading {} BLOBS", missing_blob_ids.len());

        Ok(())
    }
}
