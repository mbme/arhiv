use anyhow::{ensure, Result};

use rs_utils::log;

use crate::entities::{Changeset, ChangesetResponse, Document};
use crate::prime_server::PrimeServerRPC;

use super::db::{ArhivConnection, SETTING_ARHIV_ID, SETTING_LAST_SYNC_TIME};
use super::Arhiv;

impl Arhiv {
    pub(crate) fn apply_changeset(
        &self,
        tx: &mut ArhivConnection,
        changeset: Changeset,
    ) -> Result<Vec<Document>> {
        log::debug!("applying changeset {}", &changeset);

        let schema = self.get_schema();

        let arhiv_id = tx.get_setting(&SETTING_ARHIV_ID)?;
        ensure!(
            changeset.arhiv_id == arhiv_id,
            "changeset arhiv_id {} must be equal to {}",
            changeset.arhiv_id,
            arhiv_id,
        );

        ensure!(
            changeset.data_version == self.data_version,
            "changeset data_version {} must be equal to {}",
            changeset.data_version,
            self.data_version,
        );

        ensure!(
            !tx.has_staged_documents()?,
            "there must be no staged changes"
        );

        let db_status = tx.get_db_status()?;

        ensure!(
            changeset.base_rev <= db_status.db_rev,
            "base_rev {} is greater than prime rev {}",
            changeset.base_rev,
            db_status.db_rev,
        );

        let mut conflicts = vec![];

        if changeset.is_empty() {
            log::debug!("empty changeset, ignoring");
            return Ok(conflicts);
        }

        let new_rev = db_status.db_rev.inc();
        log::debug!(
            "current rev is {}, new rev is {}",
            db_status.db_rev,
            new_rev
        );

        for mut document in changeset.documents {
            if tx.has_snapshot(&document.id, document.rev)? {
                log::warn!("Got duplicate snapshot of the {}, ignoring", &document);

                continue;
            }

            if document.is_erased() {
                document.rev = new_rev;

                tx.put_document(&document)?;

                // erase history of erased documents
                tx.erase_document_history(&document.id)?;

                continue;
            }

            match tx.get_last_snapshot(&document.id)? {
                // on conflict
                Some(prev_snapshot) if prev_snapshot.rev != document.prev_rev => {
                    if prev_snapshot.is_erased() {
                        log::warn!(
                            "Got an update for erased document {}, ignoring",
                            &document.id
                        );
                        continue;
                    }

                    if document.data != prev_snapshot.data {
                        log::warn!("Got data conflict on document {}", &document.id);
                        conflicts.push(document);
                        continue;
                    }
                }
                _ => {}
            }

            document.rev = new_rev;

            tx.put_document(&document)?;

            for blob_id in document.extract_refs(schema)?.blobs {
                let blob = tx.get_blob(&blob_id);

                ensure!(
                    blob.exists()?,
                    "Document {} references unknown blob {}",
                    &document.id,
                    &blob_id,
                );
            }
        }

        log::debug!("successfully applied a changeset");

        Ok(conflicts)
    }

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
            let tx = self.get_tx()?;
            tx.set_setting(&SETTING_LAST_SYNC_TIME, &chrono::Utc::now())?;
            tx.commit()?;
        }

        self.cleanup()?;

        Ok(())
    }

    fn sync_locally(&self) -> Result<()> {
        log::info!("Initiating local sync");

        let mut tx = self.get_tx()?;

        let changeset = tx.generate_changeset()?;
        log::debug!("prepared a changeset {}", changeset);

        tx.delete_local_staged_changes()?;

        self.apply_changeset(&mut tx, changeset)?;

        tx.commit()?;

        Ok(())
    }

    async fn sync_remotely(&self) -> Result<()> {
        log::info!("Initiating remote sync");

        let mut tx = self.get_tx()?;

        let changeset = tx.generate_changeset()?;
        let new_blob_ids = tx.get_new_blob_ids()?;
        log::debug!(
            "sync_remotely: starting {}, {} new blobs",
            &changeset,
            new_blob_ids.len()
        );

        let last_update_time = tx.get_last_update_time()?;

        let prime_rpc =
            PrimeServerRPC::new(&self.config.prime_url, &self.path_manager.downloads_dir)?;

        // TODO parallel file upload
        for blob_id in new_blob_ids {
            let blob = self.get_blob(&blob_id)?;

            prime_rpc.upload_blob(&blob).await?;
        }

        let response: ChangesetResponse = prime_rpc.send_changeset(&changeset).await?;

        log::debug!("sync_remotely: got response {}", &response);

        ensure!(
            last_update_time == self.get_connection()?.get_last_update_time()?,
            "last_update_time must not change",
        );

        tx.delete_local_staged_changes()?;
        tx.apply_changeset_response(response)?;

        tx.commit()?;

        log::debug!("sync_remotely: success!");

        Ok(())
    }

    async fn download_missing_blobs(&self) -> Result<()> {
        let missing_blob_ids = self.get_connection()?.get_missing_blob_ids()?;

        log::debug!("There are {} missing local BLOBs", missing_blob_ids.len());

        if missing_blob_ids.is_empty() {
            return Ok(());
        }

        let prime_rpc =
            PrimeServerRPC::new(&self.config.prime_url, &self.path_manager.downloads_dir)?;

        // TODO parallel file download
        for blob_id in missing_blob_ids {
            let blob = self.get_blob(&blob_id)?;

            prime_rpc.download_blob(&blob).await?;
        }

        Ok(())
    }
}
