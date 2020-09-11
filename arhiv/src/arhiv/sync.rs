use super::Arhiv;
use crate::entities::*;
use crate::storage::*;
use anyhow::*;
use reqwest::Client;
use rs_utils::{ensure_file_exists, read_file_as_stream, FsTransaction};

impl Arhiv {
    pub(super) fn apply_changeset(&self, changeset: Changeset, delete_staged: bool) -> Result<()> {
        log::debug!("applying changeset {}", &changeset);

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.get_tx()?;

        let rev = tx.get_rev()?;

        if changeset.base_rev > rev {
            bail!(
                "base_rev {} is greater than prime rev {}",
                changeset.base_rev,
                rev
            );
        }

        if changeset.is_empty() {
            return Ok(());
        }

        let new_rev = rev + 1;

        for mut document in changeset.documents {
            // FIXME merge documents
            document.rev = new_rev;
            tx.put_document(&document)?;
            tx.put_document_history(&document)?;
        }

        if delete_staged {
            tx.delete_staged_documents()?;
        }

        let mut fs_tx = FsTransaction::new();
        for mut attachment in changeset.attachments {
            // save attachment
            attachment.rev = new_rev;
            tx.put_attachment(&attachment)?;

            let attachment_data = self.get_attachment_data(&attachment.id);
            let file_path = attachment_data.get_staged_file_path();
            ensure_file_exists(&file_path)
                .context(anyhow!("Attachment data for {} is missing", &attachment.id))?;

            // save attachment file
            fs_tx.move_file(
                file_path.to_string(),
                attachment_data.get_committed_file_path(),
            )?;
        }

        tx.commit()?;
        fs_tx.commit();
        log::debug!("successfully applied a changeset");

        Ok(())
    }

    pub(super) fn generate_changeset_response(
        &self,
        base_rev: Revision,
    ) -> Result<ChangesetResponse> {
        let conn = self.storage.get_connection()?;

        let documents = conn.get_documents_since(base_rev + 1)?;
        let attachments = conn.get_attachments_since(base_rev + 1)?;

        Ok(ChangesetResponse {
            latest_rev: conn.get_rev()?,
            base_rev,
            documents,
            attachments,
        })
    }

    pub async fn sync(&self) -> Result<()> {
        let conn = self.storage.get_connection()?;

        let is_prime = conn.is_prime()?;
        let changeset = conn.get_changeset()?;

        if is_prime {
            self.sync_locally(changeset)
        } else {
            self.sync_remotely(changeset).await
        }
    }

    fn sync_locally(&self, changeset: Changeset) -> Result<()> {
        self.apply_changeset(changeset, true)
    }

    async fn sync_remotely(&self, changeset: Changeset) -> Result<()> {
        log::debug!("sync_remotely: starting {}", &changeset);

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.get_tx()?;

        // TODO parallel file upload
        for attachment in changeset.attachments.iter() {
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
            .post(&self.config.get_changeset_url()?)
            .json(&changeset)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .parse()?;

        log::debug!("sync_remotely: got response {}", &response);

        let rev = tx.get_rev()?;

        if response.base_rev != rev {
            bail!("base_rev isn't equal to current rev");
        }

        for document in response.documents {
            tx.put_document(&document)?;
        }
        tx.delete_staged_documents()?;

        let mut fs_tx = FsTransaction::new();
        for attachment in response.attachments {
            tx.put_attachment(&attachment)?;

            // if we've sent few attachments, move them to committed data directory
            if changeset.contains_attachment(&attachment.id) {
                let attachment_data = self.get_attachment_data(&attachment.id);

                fs_tx.move_file(
                    attachment_data.get_staged_file_path(),
                    attachment_data.get_committed_file_path(),
                )?;
            }
        }

        fs_tx.commit();
        tx.commit()?;

        log::debug!("sync_remotely: success!");

        Ok(())
    }
}
