use anyhow::*;

use rs_utils::log;

use super::db::*;
use super::Arhiv;
use crate::entities::*;

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
                    if prev_snapshot.is_tombstone() {
                        log::warn!("Got update for a tombstone {}, ignoring", &document.id);
                        continue;
                    }

                    if document.data != prev_snapshot.data {
                        log::warn!("Got data conflict on document {}", &document.id);
                        conflicts.push(document);
                        continue;
                    }

                    if document.refs != prev_snapshot.refs {
                        log::warn!("Got refs conflict on document {}", &document.id);
                        conflicts.push(document);
                        continue;
                    }
                }
                _ => {}
            }

            document.rev = new_rev;

            tx.put_document(&document)?;

            // erase history of deleted documents
            if document.is_tombstone() {
                tx.erase_document_history(&document.id)?;
            }

            // check if attachment data is available
            if Attachment::is_attachment(&document) {
                let attachment = Attachment::from(document)?;
                let hash = attachment.get_hash();
                let attachment_data = tx.get_attachment_data(hash);

                ensure!(
                    attachment_data.exists()?,
                    "Attachment data {} for attachment {} is missing",
                    &attachment_data.hash,
                    &attachment.id
                );
            }
        }

        log::debug!("successfully applied a changeset");

        Ok(conflicts)
    }

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

            // erase history of deleted documents
            if document.is_tombstone() {
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

    pub(crate) fn generate_changeset_response(
        &self,
        tx: &ArhivTransaction,
        base_rev: Revision,
        conflicts: Vec<Document>,
    ) -> Result<ChangesetResponse> {
        let next_rev = base_rev.inc();
        let new_snapshots = tx.get_new_snapshots_since(&next_rev)?;

        let arhiv_id = tx.get_setting(SETTING_ARHIV_ID)?;
        let latest_rev = tx.get_db_rev()?;

        Ok(ChangesetResponse {
            arhiv_id,
            latest_rev,
            base_rev,
            new_snapshots,
            conflicts,
        })
    }

    fn prepare_changeset(&self, tx: &ArhivTransaction) -> Result<Changeset> {
        let db_status = tx.get_db_status()?;

        tx.delete_unused_local_attachments()?;

        let documents = tx.list_documents(DOCUMENT_FILTER_STAGED)?.items;

        let changeset = Changeset {
            arhiv_id: tx.get_setting(SETTING_ARHIV_ID)?,
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

        let changeset = {
            let tx = self.db.get_tx()?;
            let changeset = self.prepare_changeset(&tx)?;

            tx.commit()?;

            changeset
        };
        log::debug!("sync_remotely: starting {}", &changeset);

        let last_update_time = self.db.get_connection()?.get_last_update_time()?;

        let network_service = self.get_network_service()?;
        // TODO parallel file upload
        for attachment in changeset
            .documents
            .iter()
            .filter(|document| Attachment::is_attachment(document))
        {
            let attachment = Attachment::from(attachment.clone())?;
            let hash = attachment.get_hash();
            let attachment_data = self.get_attachment_data(hash)?;

            network_service
                .upload_attachment_data(&attachment_data)
                .await?;
        }

        let response: ChangesetResponse = network_service.send_changeset(&changeset).await?;

        log::debug!("sync_remotely: got response {}", &response);

        ensure!(
            last_update_time == self.db.get_connection()?.get_last_update_time()?,
            "last_update_time must not change",
        );

        let mut tx = self.db.get_tx()?;

        tx.delete_local_staged_changes()?;
        self.apply_changeset_response(&mut tx, response)?;

        tx.commit()?;

        log::debug!("sync_remotely: success!");

        Ok(())
    }
}
