use super::Arhiv;
use crate::entities::*;
use crate::fs_transaction::FsTransaction;
use crate::storage::*;
use anyhow::*;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fs::File;

impl Arhiv {
    // FIXME use parent transaction
    fn apply_changeset(
        &self,
        changeset: Changeset,
        attachment_data: HashMap<Id, String>,
        local: bool,
    ) -> Result<()> {
        log::debug!(
            "applying {} changeset {}",
            if local { "local" } else { "remote" },
            &changeset
        );

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

        if !local && conn.has_staged_changes()? {
            return Err(anyhow!("can't apply changes: there are staged changes"));
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

    fn get_changeset_response(&self, base_rev: Revision) -> Result<ChangesetResponse> {
        let conn = self.storage.get_connection()?;

        let mut filter = QueryFilter::default();
        filter.page_size = None; // fetch all items

        let documents = conn.get_documents(base_rev + 1, filter)?;
        let attachments = conn.get_committed_attachments_with_rev(base_rev + 1)?;

        Ok(ChangesetResponse {
            latest_rev: conn.get_rev()?,
            base_rev,
            documents,
            attachments,
        })
    }

    pub fn commit(&self) -> Result<()> {
        if self.config.prime {
            self.commit_locally()
        } else {
            self.commit_remotely()
        }
    }

    fn commit_remotely(&self) -> Result<()> {
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

    fn commit_locally(&self) -> Result<()> {
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

        self.apply_changeset(changeset, attachment_data, true)?;

        {
            let mut conn = self.storage.get_writable_connection()?;
            let conn = conn.get_tx()?;

            conn.delete_staged_documents()?;
            conn.delete_staged_attachments()?;

            conn.commit()?;
        }

        Ok(())
    }

    pub fn exchange(
        &self,
        changeset: Changeset,
        attachment_data: HashMap<String, String>,
    ) -> Result<ChangesetResponse> {
        if !self.config.prime {
            return Err(anyhow!("can't exchange: not a prime"));
        }

        let base_rev = changeset.base_rev.clone();

        self.apply_changeset(changeset, attachment_data, false)?;

        self.get_changeset_response(base_rev)
    }
}
