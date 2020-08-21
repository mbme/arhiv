use super::Arhiv;
use crate::entities::*;
use crate::fs_transaction::FsTransaction;
use crate::storage::*;
use anyhow::*;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fs::File;

impl Arhiv {
    pub(super) fn apply_changeset(
        &self,
        changeset: Changeset,
        attachment_data: HashMap<Id, String>,
    ) -> Result<()> {
        log::debug!("applying changeset {}", &changeset);

        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;

        let rev = conn.get_rev()?;

        if changeset.base_rev > rev {
            return Err(anyhow!(
                "base_rev {} is greater than prime rev {}",
                changeset.base_rev,
                rev
            ));
        }

        if changeset.is_empty() {
            return Ok(());
        }

        let new_rev = rev + 1;

        for mut document in changeset.documents {
            // FIXME merge documents
            document.rev = new_rev;
            conn.put_document(&document)?;
        }

        let mut fs_tx = FsTransaction::new();
        for mut attachment in changeset.attachments {
            // save attachment
            attachment.rev = new_rev;
            conn.put_attachment(&attachment)?;

            let file_path = attachment_data
                .get(&attachment.id)
                .ok_or(anyhow!("Attachment data for {} is missing", &attachment.id))?;

            // save attachment file
            fs_tx.move_file(
                file_path.to_string(),
                self.storage
                    .get_committed_attachment_file_path(&attachment.id),
            )?;
        }

        conn.commit()?;
        fs_tx.commit();
        log::debug!("successfully applied a changeset");

        Ok(())
    }

    pub(super) fn get_changeset_response(&self, base_rev: Revision) -> Result<ChangesetResponse> {
        let conn = self.storage.get_connection()?;

        let mut document_filter = DocumentFilter::default();
        // fetch all items
        document_filter.page_size = None;
        document_filter.skip_archived = None;
        let documents = conn.get_documents(base_rev + 1, document_filter)?;

        let mut attachment_filter = AttachmentFilter::default();
        // fetch all items
        attachment_filter.page_size = None;
        let attachments = conn.get_attachments(base_rev + 1, attachment_filter)?;

        Ok(ChangesetResponse {
            latest_rev: conn.get_rev()?,
            base_rev,
            documents,
            attachments,
        })
    }

    pub fn sync(&self) -> Result<()> {
        if self.config.is_prime {
            self.sync_locally()
        } else {
            self.sync_remotely()
        }
    }

    fn sync_remotely(&self) -> Result<()> {
        let primary_url = self
            .config
            .primary_url
            .as_ref()
            .ok_or(anyhow!("can't sync: primary_url is missing"))?;

        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;

        let changeset = conn.get_changeset()?;

        for attachment in changeset.attachments.iter() {
            // FIXME async parallel upload
            let file = File::open(self.storage.get_staged_attachment_file_path(&attachment.id))?;

            let res = Client::new().post(primary_url).body(file).send()?;

            if !res.status().is_success() {
                return Err(anyhow!(
                    "failed to upload attachment data {}",
                    &attachment.id
                ));
            }
        }

        let response: ChangesetResponse = Client::new()
            .post(primary_url)
            .json(&changeset)
            .send()?
            .text()?
            .parse()?;

        let rev = conn.get_rev()?;

        if response.base_rev != rev {
            return Err(anyhow!("base_rev isn't equal to current rev"));
        }

        for document in response.documents {
            conn.put_document(&document)?;
        }

        for attachment in response.attachments {
            conn.put_attachment(&attachment)?;
        }

        conn.delete_staged_documents()?;
        conn.delete_staged_attachments()?;

        conn.commit()?;

        Ok(())
    }

    fn sync_locally(&self) -> Result<()> {
        let changeset = {
            let conn = self.storage.get_connection()?;

            conn.get_changeset()?
        };

        let mut attachment_data = HashMap::new();
        for attachment in &changeset.attachments {
            attachment_data.insert(
                attachment.id.clone(),
                self.storage.get_staged_attachment_file_path(&attachment.id),
            );
        }

        self.apply_changeset(changeset, attachment_data)?;

        {
            let mut conn = self.storage.get_writable_connection()?;
            let conn = conn.get_tx()?;

            conn.delete_staged_documents()?;
            conn.delete_staged_attachments()?;

            conn.commit()?;
        }

        Ok(())
    }
}
