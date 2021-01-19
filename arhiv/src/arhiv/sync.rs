use super::Arhiv;
use crate::entities::*;
use crate::storage::*;
use anyhow::*;
use reqwest::Client;
use rs_utils::{ensure_file_exists, read_file_as_stream, FsTransaction};

impl Arhiv {
    pub(crate) fn apply_changeset(&self, changeset: Changeset) -> Result<()> {
        log::debug!("applying changeset {}", &changeset);

        ensure!(
            changeset.arhiv_id == self.config.get_arhiv_id(),
            "changeset arhiv_id {} must be equal to {}",
            changeset.arhiv_id,
            self.config.get_arhiv_id()
        );

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.get_tx()?;

        let rev = tx.get_rev()?;

        ensure!(
            changeset.base_rev <= rev,
            "base_rev {} is greater than prime rev {}",
            changeset.base_rev,
            rev
        );

        let schema_version = tx.get_schema_version()?;

        ensure!(
            schema_version == changeset.schema_version,
            "db schema version {} is different from changeset version {}",
            schema_version,
            changeset.schema_version,
        );

        if changeset.is_empty() {
            log::debug!("empty changeset, ignoring");
            return Ok(());
        }

        let new_rev = rev.inc();
        log::debug!("current rev is {}, new rev is {}", rev, new_rev);

        let mut fs_tx = FsTransaction::new();

        for mut document in changeset.documents {
            document.rev = new_rev.clone();

            tx.put_document(&document)?;
            tx.put_document_history(&document, &changeset.base_rev)?;

            if document.is_attachment() {
                let attachment_data = self.get_attachment_data(&document.id);
                let file_path = attachment_data.get_staged_file_path();
                ensure_file_exists(&file_path)
                    .context(anyhow!("Attachment data for {} is missing", &document.id))?;

                // save attachment file
                fs_tx.move_file(
                    file_path.to_string(),
                    attachment_data.get_committed_file_path(),
                )?;
            }
        }

        tx.set_setting(DbSettings::DbRevision, new_rev.to_string())?;

        tx.commit()?;
        fs_tx.commit();
        log::debug!("successfully applied a changeset");

        Ok(())
    }

    pub(crate) fn generate_changeset_response(
        &self,
        base_rev: Revision,
    ) -> Result<ChangesetResponse> {
        let conn = self.storage.get_connection()?;

        let next_rev = base_rev.inc();
        let documents = conn.get_documents_since(&next_rev)?;

        Ok(ChangesetResponse {
            arhiv_id: conn.get_arhiv_id()?,
            latest_rev: conn.get_rev()?,
            base_rev,
            documents,
        })
    }

    pub async fn sync(&self) -> Result<()> {
        let conn = self.storage.get_connection()?;
        let is_prime = conn.is_prime()?;

        log::info!(
            "Initiating {} sync",
            if is_prime { "local" } else { "remote" }
        );

        let changeset = Changeset {
            arhiv_id: self.config.get_arhiv_id().to_string(),
            schema_version: conn.get_schema_version()?,
            base_rev: conn.get_rev()?,
            documents: conn.list_documents(DOCUMENT_FILTER_STAGED)?.items,
        };
        log::debug!("prepared a changeset {}", changeset);

        let result = if is_prime {
            self.sync_locally(changeset)
        } else {
            self.sync_remotely(changeset).await
        };

        if let Err(ref err) = result {
            log::error!(
                "sync failed on {}: {}",
                if is_prime { "prime" } else { "replica" },
                err
            );
        } else {
            log::info!("sync succeeded");
        }

        result
    }

    fn sync_locally(&self, changeset: Changeset) -> Result<()> {
        self.apply_changeset(changeset)?;

        // make sure there are no more staged documents
        assert_eq!(self.storage.get_connection()?.count_documents()?.1, 0);

        Ok(())
    }

    async fn sync_remotely(&self, changeset: Changeset) -> Result<()> {
        log::debug!("sync_remotely: starting {}", &changeset);

        let last_update_time = self.storage.get_connection()?.get_last_update_time()?;

        // TODO parallel file upload
        for attachment in changeset
            .documents
            .iter()
            .filter(|document| document.is_attachment())
        {
            let data = self.get_attachment_data(&attachment.id);

            let file_path = data.get_staged_file_path();

            log::debug!(
                "sync_remotely: uploading attachment {} ({})",
                &attachment.id,
                &file_path
            );

            let file_stream = read_file_as_stream(&file_path).await?;

            Client::new()
                .post(&data.get_url()?)
                .body(reqwest::Body::wrap_stream(file_stream))
                .send()
                .await?
                .error_for_status()?;
        }

        log::debug!("sync_remotely: sending changeset...");
        let response: ChangesetResponse = Client::new()
            .post(&format!("{}/changeset", self.config.get_prime_url()?))
            .json(&changeset)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .parse()?;

        log::debug!("sync_remotely: got response {}", &response);

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.get_tx()?;

        let arhiv_id = tx.get_arhiv_id()?;
        let rev = tx.get_rev()?;

        ensure!(
            response.arhiv_id == arhiv_id,
            "changeset response arhiv_id {} isn't equal to current arhiv_id {}",
            response.arhiv_id,
            arhiv_id,
        );
        ensure!(
            response.base_rev == rev,
            "base_rev {} isn't equal to current rev {}",
            response.base_rev,
            rev,
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
                let attachment_data = self.get_attachment_data(&document.id);

                fs_tx.move_file(
                    attachment_data.get_staged_file_path(),
                    attachment_data.get_committed_file_path(),
                )?;
            }
        }

        tx.set_setting(DbSettings::DbRevision, response.latest_rev.to_string())?;

        // make sure there are no more staged documents
        assert_eq!(tx.count_documents()?.1, 0);

        fs_tx.commit();
        tx.commit()?;

        log::debug!("sync_remotely: success!");

        Ok(())
    }
}
