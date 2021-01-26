use super::Arhiv;
use crate::db::*;
use crate::entities::*;
use anyhow::*;
use rs_utils::FsTransaction;
use tracing::{debug, error, info};

impl Arhiv {
    pub(crate) fn apply_changeset(&self, changeset: Changeset) -> Result<()> {
        debug!("applying changeset {}", &changeset);

        ensure!(
            changeset.arhiv_id == self.config.get_arhiv_id(),
            "changeset arhiv_id {} must be equal to {}",
            changeset.arhiv_id,
            self.config.get_arhiv_id()
        );

        let mut conn = self.db.get_writable_connection()?;
        let tx = conn.get_tx()?;

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
            debug!("empty changeset, ignoring");
            return Ok(());
        }

        let new_rev = db_status.db_rev.inc();
        debug!(
            "current rev is {}, new rev is {}",
            db_status.db_rev, new_rev
        );

        let mut fs_tx = FsTransaction::new();

        for mut document in changeset.documents {
            document.rev = new_rev.clone();

            tx.put_document(&document)?;
            tx.put_document_history(&document, &changeset.base_rev)?;

            if document.is_attachment() {
                ensure!(
                    self.data_service.staged_file_exists(&document.id)?,
                    "Attachment data for {} is missing",
                    &document.id
                );

                // double-check file integrity
                let expected_hash = self.schema.get_field_string(&document, "hash")?;
                let hash = self.data_service.get_staged_file_hash(&document.id)?;
                ensure!(
                    hash == expected_hash,
                    "Attachment {} data is corrupted: hash doesn't match",
                    &document.id
                );

                // save attachment file
                fs_tx.move_file(
                    self.data_service.get_staged_file_path(&document.id),
                    self.data_service.get_committed_file_path(&document.id),
                )?;
            }
        }

        tx.put_db_status(DbStatus {
            db_rev: new_rev,
            last_sync_time: chrono::Utc::now(),
            ..db_status
        })?;

        tx.commit()?;
        fs_tx.commit();
        debug!("successfully applied a changeset");

        Ok(())
    }

    pub(crate) fn generate_changeset_response(
        &self,
        base_rev: Revision,
    ) -> Result<ChangesetResponse> {
        let conn = self.db.get_connection()?;

        let next_rev = base_rev.inc();
        let documents = conn.get_documents_since(&next_rev)?;

        let db_status = conn.get_db_status()?;

        Ok(ChangesetResponse {
            arhiv_id: db_status.arhiv_id,
            latest_rev: db_status.db_rev,
            base_rev,
            documents,
        })
    }

    pub async fn sync(&self) -> Result<()> {
        let conn = self.db.get_connection()?;

        let db_status = conn.get_db_status()?;

        info!(
            "Initiating {} sync",
            if db_status.is_prime {
                "local"
            } else {
                "remote"
            }
        );

        let changeset = Changeset {
            arhiv_id: self.config.get_arhiv_id().to_string(),
            schema_version: db_status.schema_version.clone(),
            base_rev: db_status.db_rev.clone(),
            documents: conn.list_documents(DOCUMENT_FILTER_STAGED)?.items,
        };
        debug!("prepared a changeset {}", changeset);

        let result = if db_status.is_prime {
            self.sync_locally(changeset)
        } else {
            self.sync_remotely(changeset).await
        };

        if let Err(ref err) = result {
            error!("sync failed on {}: {}", db_status.get_prime_status(), err);
        } else {
            info!("sync succeeded");
        }

        result
    }

    fn sync_locally(&self, changeset: Changeset) -> Result<()> {
        self.apply_changeset(changeset)?;

        // make sure there are no more staged documents
        assert_eq!(self.db.get_connection()?.count_documents()?.1, 0);

        Ok(())
    }

    async fn sync_remotely(&self, changeset: Changeset) -> Result<()> {
        debug!("sync_remotely: starting {}", &changeset);

        let last_update_time = self.db.get_connection()?.get_last_update_time()?;

        let network_service = self.get_network_service()?;
        // TODO parallel file upload
        for attachment in changeset
            .documents
            .iter()
            .filter(|document| document.is_attachment())
        {
            network_service
                .upload_attachment_data(&attachment.id)
                .await?;
        }

        let response: ChangesetResponse = network_service.send_changeset(&changeset).await?;

        debug!("sync_remotely: got response {}", &response);

        let mut conn = self.db.get_writable_connection()?;
        let tx = conn.get_tx()?;

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
        ensure!(
            last_update_time == tx.get_last_update_time()?,
            "last_update_time must not change",
        );

        let mut fs_tx = FsTransaction::new();

        for document in response.documents {
            tx.put_document(&document)?;

            // if we've sent any attachments, move them to committed data directory
            if document.is_attachment() && changeset.contains(&document.id) {
                fs_tx.move_file(
                    self.data_service.get_staged_file_path(&document.id),
                    self.data_service.get_committed_file_path(&document.id),
                )?;
            }
        }

        tx.put_db_status(DbStatus {
            db_rev: response.latest_rev,
            last_sync_time: chrono::Utc::now(),
            ..db_status
        })?;

        // make sure there are no more staged documents
        ensure!(
            tx.count_documents()?.1 == 0,
            "There are staged documents after remote sync"
        );

        fs_tx.commit();
        tx.commit()?;

        debug!("sync_remotely: success!");

        Ok(())
    }
}
