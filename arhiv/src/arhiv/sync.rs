use std::collections::HashSet;

use super::db::*;
use super::Arhiv;
use crate::entities::*;
use anyhow::*;
use rs_utils::{
    log::{debug, error, info, warn},
    FsTransaction,
};

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
            document.rev = new_rev;

            // handle deleted documents
            if document.is_deleted() {
                // remove attachment data if needed
                match self.get_document(&document.id)? {
                    Some(original_document) if Attachment::is_attachment(&original_document) => {
                        let attachment = Attachment::from(original_document)?;
                        let hash = attachment.get_hash();

                        self.blob_manager.remove_attachment_data(&mut fs_tx, &hash);
                        debug!(
                            "removing attachment data for deleted document {}",
                            &attachment.id
                        );
                    }
                    _ => {}
                };

                tx.put_document_history(&document, &changeset.base_rev)?;
                tx.erase_document_history(&document.id)?;

                continue;
            }

            tx.put_document_history(&document, &changeset.base_rev)?;

            if Attachment::is_attachment(&document) {
                let attachment = Attachment::from(document)?;
                let hash = attachment.get_hash();
                let attachment_data = self.get_attachment_data(hash);

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

        fs_tx.commit()?;
        tx.commit()?;
        debug!("successfully applied a changeset");

        Ok(())
    }

    fn apply_changeset_response(&self, response: ChangesetResponse) -> Result<()> {
        let mut conn = self.db.get_writable_connection()?;
        let tx = conn.get_tx()?;

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

        let mut fs_tx = FsTransaction::new();

        for document_history in response.documents_history {
            let document = &document_history.document;

            // handle deleted documents
            if document.is_deleted() {
                // remove attachment data if needed
                match self.get_document(&document.id)? {
                    Some(original_document) if Attachment::is_attachment(&original_document) => {
                        let attachment = Attachment::from(original_document)?;
                        let hash = attachment.get_hash();

                        self.blob_manager.remove_attachment_data(&mut fs_tx, &hash);
                        debug!(
                            "removing attachment data for deleted document {}",
                            &attachment.id
                        );
                    }
                    _ => {}
                };

                tx.put_document_history(&document_history.document, &document_history.base_rev)?;
                tx.erase_document_history(&document.id)?;

                continue;
            }

            tx.put_document_history(&document_history.document, &document_history.base_rev)?;
        }

        // copy documents updated since base_rev into documents table
        tx.copy_documents_from_history(&response.base_rev)?;

        tx.set_setting(SETTING_DB_REV, response.latest_rev)?;

        fs_tx.commit()?;
        tx.commit()?;
        debug!("successfully applied a changeset response");

        Ok(())
    }

    pub(crate) fn generate_changeset_response(
        &self,
        base_rev: Revision,
    ) -> Result<ChangesetResponse> {
        let conn = self.db.get_connection()?;

        let next_rev = base_rev.inc();
        let documents_history = conn.get_documents_history_since(&next_rev)?;

        let arhiv_id = conn.get_setting(SETTING_ARHIV_ID)?;
        let latest_rev = conn.get_setting(SETTING_DB_REV)?;

        Ok(ChangesetResponse {
            arhiv_id,
            latest_rev,
            base_rev,
            documents_history,
        })
    }

    fn generate_changeset(&self) -> Result<(Changeset, Vec<Document>)> {
        let conn = self.db.get_connection()?;

        let db_status = conn.get_db_status()?;

        let documents = conn.list_documents(DOCUMENT_FILTER_STAGED)?.items;

        // collect ids in use
        let mut refs: HashSet<Id> = HashSet::new();
        for document in documents.iter() {
            for id in document.refs.iter() {
                refs.insert(id.clone());
            }
        }

        let mut documents_in_use = Vec::new();
        let mut unused_attachments = Vec::new();

        for document in documents {
            let is_unused_attachment = Attachment::is_attachment(&document)
                // skip attachments which were created before last sync
                && document.created_at > db_status.last_sync_time
                // attachments which aren't in use
                && !refs.contains(&document.id);

            if is_unused_attachment {
                unused_attachments.push(document);
            } else {
                documents_in_use.push(document);
            }
        }

        let changeset = Changeset {
            arhiv_id: self.config.get_arhiv_id().to_string(),
            schema_version: db_status.schema_version.clone(),
            base_rev: db_status.db_rev,
            documents: documents_in_use,
        };

        Ok((changeset, unused_attachments))
    }

    pub async fn sync(&self) -> Result<()> {
        let db_status = self.db.get_connection()?.get_db_status()?;

        info!(
            "Initiating {} sync",
            if db_status.is_prime {
                "local"
            } else {
                "remote"
            }
        );

        let (changeset, unused_attachments) = self.generate_changeset()?;
        debug!("prepared a changeset {}", changeset);

        let result = if db_status.is_prime {
            self.sync_locally(changeset)
        } else {
            self.sync_remotely(changeset).await
        };

        if let Err(ref err) = result {
            error!("sync failed on {}: {}", db_status.get_prime_status(), err);

            return result;
        } else {
            info!("sync succeeded");
        }

        // update last sync time
        self.db
            .get_writable_connection()?
            .set_setting(SETTING_LAST_SYNC_TIME, chrono::Utc::now())?;

        // remove unused local attachments
        if !unused_attachments.is_empty() {
            warn!(
                "removing {} unused attachments after sync",
                unused_attachments.len()
            );

            let mut conn = self.db.get_writable_connection()?;
            let tx = conn.get_tx()?;
            let mut fs_tx = FsTransaction::new();

            for document in unused_attachments {
                tx.delete_document(&document.id)?;

                let hash = Attachment::from(document)?.get_hash();
                self.blob_manager.remove_attachment_data(&mut fs_tx, &hash);
            }

            tx.commit()?;
            fs_tx.commit()?;
        }

        // clean up the db
        self.db.get_writable_connection()?.vacuum()?;

        Ok(())
    }

    fn sync_locally(&self, changeset: Changeset) -> Result<()> {
        self.apply_changeset(changeset)?;

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
            .filter(|document| Attachment::is_attachment(document))
        {
            let attachment = Attachment::from(attachment.clone())?;
            let hash = attachment.get_hash();
            let attachment_data = self.get_attachment_data(hash);

            network_service
                .upload_attachment_data(&attachment_data)
                .await?;
        }

        let response: ChangesetResponse = network_service.send_changeset(&changeset).await?;

        debug!("sync_remotely: got response {}", &response);

        ensure!(
            last_update_time == self.db.get_connection()?.get_last_update_time()?,
            "last_update_time must not change",
        );

        self.apply_changeset_response(response)?;

        debug!("sync_remotely: success!");

        Ok(())
    }
}
