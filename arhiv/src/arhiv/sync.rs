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
    ) -> Result<()> {
        log::debug!("applying changeset {}", &changeset);

        ensure!(
            changeset.arhiv_id == self.config.get_arhiv_id(),
            "changeset arhiv_id {} must be equal to {}",
            changeset.arhiv_id,
            self.config.get_arhiv_id()
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

        ensure!(
            db_status.schema_version == changeset.schema_version,
            "db schema version {} is different from changeset version {}",
            db_status.schema_version,
            changeset.schema_version,
        );

        if changeset.is_empty() {
            log::debug!("empty changeset, ignoring");
            return Ok(());
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

            document.rev = new_rev;

            tx.put_document_history(&document, &changeset.base_rev)?;

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

        // copy documents updated since base_rev into documents table
        tx.copy_documents_from_history(&changeset.base_rev)?;

        tx.set_setting(SETTING_DB_REV, new_rev)?;

        log::debug!("successfully applied a changeset");

        Ok(())
    }

    fn apply_changeset_response(
        &self,
        tx: &mut ArhivTransaction,
        response: ChangesetResponse,
    ) -> Result<()> {
        ensure!(
            !tx.has_staged_documents()?,
            "there must be no staged changes"
        );

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

        for document_history in response.documents_history {
            let document = &document_history.document;

            tx.put_document_history(&document_history.document, &document_history.base_rev)?;

            // erase history of deleted documents
            if document.is_tombstone() {
                tx.erase_document_history(&document.id)?;
            }
        }

        // copy documents updated since base_rev into documents table
        tx.copy_documents_from_history(&response.base_rev)?;

        tx.set_setting(SETTING_DB_REV, response.latest_rev)?;

        log::debug!("successfully applied a changeset response");

        Ok(())
    }

    pub(crate) fn generate_changeset_response(
        &self,
        tx: &ArhivTransaction,
        base_rev: Revision,
    ) -> Result<ChangesetResponse> {
        let next_rev = base_rev.inc();
        let documents_history = tx.get_documents_history_since(&next_rev)?;

        let arhiv_id = tx.get_setting(SETTING_ARHIV_ID)?;
        let latest_rev = tx.get_setting(SETTING_DB_REV)?;

        Ok(ChangesetResponse {
            arhiv_id,
            latest_rev,
            base_rev,
            documents_history,
        })
    }

    fn prepare_changeset(&self, tx: &ArhivTransaction) -> Result<Changeset> {
        let db_status = tx.get_db_status()?;

        tx.delete_unused_local_attachments()?;

        let documents = tx.list_documents(DOCUMENT_FILTER_STAGED)?.items;

        let changeset = Changeset {
            arhiv_id: self.config.get_arhiv_id().to_string(),
            schema_version: db_status.schema_version.clone(),
            base_rev: db_status.db_rev,
            documents,
        };

        Ok(changeset)
    }

    pub async fn sync(&self) -> Result<()> {
        let result = if self.config.is_prime() {
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

        self.reset_local_staged_changes(&mut tx)?;

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

        self.reset_local_staged_changes(&mut tx)?;

        self.apply_changeset_response(&mut tx, response)?;

        tx.commit()?;

        log::debug!("sync_remotely: success!");

        Ok(())
    }

    fn reset_local_staged_changes(&self, tx: &mut ArhivTransaction) -> Result<()> {
        tx.delete_local_staged_changes()?;

        let current_rev = tx.get_setting(SETTING_DB_REV)?;
        tx.copy_documents_from_history(&current_rev)?;

        Ok(())
    }
}
