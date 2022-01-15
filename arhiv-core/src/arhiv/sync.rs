use anyhow::{ensure, Result};

use rs_utils::log;

use super::db::*;
use super::Arhiv;
use crate::entities::*;
use crate::prime_server::PrimeServerRPC;

impl Arhiv {
    pub(crate) fn apply_changeset(
        &self,
        tx: &mut ArhivTransaction,
        changeset: Changeset,
    ) -> Result<Vec<Document>> {
        log::debug!("applying changeset {}", &changeset);

        let arhiv_id = tx.get_setting(SETTING_ARHIV_ID)?;
        ensure!(
            changeset.arhiv_id == arhiv_id,
            "changeset arhiv_id {} must be equal to {}",
            changeset.arhiv_id,
            arhiv_id,
        );

        ensure!(
            changeset.schema_version == self.schema.version,
            "changeset schema_version {} must be equal to {}",
            changeset.schema_version,
            self.schema.version,
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
            if tx.has_snapshot(&document.snapshot_id)? {
                log::warn!(
                    "Got duplicate snapshot {} of document {}, ignoring",
                    &document.snapshot_id,
                    &document
                );

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

            // erase history of erased documents
            if document.is_erased() {
                tx.erase_document_history(&document.id)?;
            }

            for blob_id in document.extract_refs(&self.schema)?.blobs {
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

    #[allow(clippy::unused_self)]
    fn apply_changeset_response(
        &self,
        tx: &mut ArhivTransaction,
        response: ChangesetResponse,
    ) -> Result<()> {
        let db_status = tx.get_db_status()?;

        ensure!(
            response.arhiv_id == db_status.arhiv_id,
            "changeset response arhiv_id {} isn't equal to current arhiv_id {}",
            response.arhiv_id,
            db_status.arhiv_id,
        );
        ensure!(
            response.base_rev == db_status.db_rev,
            "base_rev {} isn't equal to current rev {}",
            response.base_rev,
            db_status.db_rev,
        );

        for document in response.new_snapshots {
            tx.put_document(&document)?;

            // erase history of erased documents
            if document.is_erased() {
                tx.erase_document_history(&document.id)?;
            }
        }

        if !response.conflicts.is_empty() {
            log::warn!(
                "Got {} conflict(s) in changeset response",
                response.conflicts.len()
            );
        }

        // save conflicts in documents table
        for document in response.conflicts {
            log::warn!("Conflict in {}", &document);
            tx.put_document(&document)?;
        }

        log::debug!("successfully applied a changeset response");

        Ok(())
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn generate_changeset_response(
        &self,
        tx: &ArhivTransaction,
        base_rev: Revision,
        conflicts: Vec<Document>,
    ) -> Result<ChangesetResponse> {
        let next_rev = base_rev.inc();
        let new_snapshots = tx.get_new_snapshots_since(next_rev)?;

        let arhiv_id = tx.get_setting(SETTING_ARHIV_ID)?;
        let latest_rev = tx.get_db_rev()?;

        Ok(ChangesetResponse {
            arhiv_id,
            base_rev,
            latest_rev,
            new_snapshots,
            conflicts,
        })
    }

    fn prepare_changeset(&self, tx: &ArhivTransaction) -> Result<Changeset> {
        let db_status = tx.get_db_status()?;

        let documents = tx.list_documents(&Filter::all_staged_documents())?.items;

        let changeset = Changeset {
            schema_version: self.schema.version,
            arhiv_id: db_status.arhiv_id,
            base_rev: db_status.db_rev,
            documents,
        };

        Ok(changeset)
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

        log::info!("sync succeeded");

        // update last sync time
        {
            let tx = self.db.get_tx()?;
            tx.set_setting(SETTING_LAST_SYNC_TIME, chrono::Utc::now())?;
            tx.commit()?;
        }

        // cleanup the db
        self.db.cleanup()?;

        Ok(())
    }

    fn sync_locally(&self) -> Result<()> {
        log::info!("Initiating local sync");

        let mut tx = self.db.get_tx()?;

        let changeset = self.prepare_changeset(&tx)?;
        log::debug!("prepared a changeset {}", changeset);

        tx.delete_local_staged_changes()?;

        self.apply_changeset(&mut tx, changeset)?;

        tx.commit()?;

        Ok(())
    }

    async fn sync_remotely(&self) -> Result<()> {
        log::info!("Initiating remote sync");

        let mut tx = self.db.get_tx()?;

        let changeset = self.prepare_changeset(&tx)?;
        let new_blob_ids = tx.get_new_blob_ids()?;
        log::debug!(
            "sync_remotely: starting {}, {} new blobs",
            &changeset,
            new_blob_ids.len()
        );

        let last_update_time = tx.get_last_update_time()?;

        let prime_rpc = PrimeServerRPC::new(&self.config.prime_url)?;

        // TODO parallel file upload
        for blob_id in new_blob_ids {
            let blob = self.get_blob(&blob_id)?;

            prime_rpc.upload_blob(&blob).await?;
        }

        let response: ChangesetResponse = prime_rpc.send_changeset(&changeset).await?;

        log::debug!("sync_remotely: got response {}", &response);

        ensure!(
            last_update_time == self.db.get_connection()?.get_last_update_time()?,
            "last_update_time must not change",
        );

        tx.delete_local_staged_changes()?;
        self.apply_changeset_response(&mut tx, response)?;

        tx.commit()?;

        log::debug!("sync_remotely: success!");

        Ok(())
    }
}
