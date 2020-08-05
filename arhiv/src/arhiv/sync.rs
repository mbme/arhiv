use super::storage::*;
use super::Arhiv;
use crate::entities::*;
use crate::utils::FsTransaction;
use anyhow::*;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fs::File;

impl Arhiv {
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
        let tx = conn.transaction()?;

        let rev = get_rev(&tx)?;

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

        if !local && has_staged_changes(&tx)? {
            return Err(anyhow!("can't apply changes: there are staged changes"));
        }

        let new_rev = rev + 1;

        for mut document in changeset.documents {
            // FIXME merge documents
            document.rev = new_rev;
            put_document(&tx, &document)?;
        }

        let mut fs_tx = FsTransaction::new();
        for mut attachment in changeset.attachments {
            // save attachment
            attachment.rev = new_rev;
            put_attachment(&tx, &attachment)?;

            let file_path = attachment_data
                .get(&attachment.id)
                .ok_or(anyhow!("Attachment data for {} is missing", &attachment.id))?;

            // FIXME check if we need this for local commits
            // save attachment file
            fs_tx.move_file(
                file_path.to_string(),
                self.storage.get_attachment_file_path(&attachment.id),
            )?;
        }

        tx.commit()?;
        fs_tx.commit();
        log::debug!("successfully applied a changeset");

        Ok(())
    }

    fn get_changeset_response(&self, base_rev: Revision) -> Result<ChangesetResponse> {
        let conn = self.storage.get_connection()?;

        let mut filter = QueryFilter::default();
        filter.page_size = None; // fetch all items

        let page = get_documents(&conn, base_rev + 1, filter)?;
        let attachments = get_commited_attachments_with_rev(&conn, base_rev + 1)?;

        Ok(ChangesetResponse {
            latest_rev: get_rev(&conn)?,
            base_rev,
            documents: page.results,
            attachments,
        })
    }

    fn apply_changeset_response(&self, response: ChangesetResponse) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;

        let rev = get_rev(&tx)?;

        if response.base_rev != rev {
            return Err(anyhow!("base_rev isn't equal to current rev"));
        }

        for document in response.documents {
            put_document(&tx, &document)?;
        }

        for attachment in response.attachments {
            put_attachment(&tx, &attachment)?;
        }

        delete_staged_documents(&tx)?;
        delete_staged_attachments(&tx)?;

        tx.commit()?;

        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        if self.config.prime {
            return Err(anyhow!("can't sync: not a replica"));
        }

        let primary_url = self
            .config
            .primary_url
            .as_ref()
            .ok_or(anyhow!("can't sync: primary_url is missing"))?;

        let changeset = {
            let mut conn = self.storage.get_connection()?;
            let tx = conn.transaction()?;

            get_changeset(&tx)?
        };

        for attachment in changeset.attachments.iter() {
            // FIXME async parallel upload
            let file = File::open(self.get_attachment_data_path(&attachment.id))?;

            let res = Client::new().post(primary_url).body(file).send()?;

            if !res.status().is_success() {
                return Err(anyhow!(
                    "failed to upload attachment data {}",
                    &attachment.id
                ));
            }
        }

        // FIXME lock database until we get a response
        let response: ChangesetResponse = Client::new()
            .post(primary_url)
            .json(&changeset)
            .send()?
            .text()?
            .parse()?;

        self.apply_changeset_response(response)
    }

    pub fn sync_locally(&self) -> Result<()> {
        if !self.config.prime {
            return Err(anyhow!("can't sync locally: not a prime"));
        }

        let changeset = {
            let mut conn = self.storage.get_connection()?;
            let tx = conn.transaction()?;

            get_changeset(&tx)?
        };

        let mut attachment_data = HashMap::new();
        for attachment in &changeset.attachments {
            attachment_data.insert(
                attachment.id.clone(),
                self.storage.get_attachment_file_path(&attachment.id),
            );
        }

        self.apply_changeset(changeset, attachment_data, true)?;

        {
            let mut conn = self.storage.get_writable_connection()?;
            let tx = conn.transaction()?;

            delete_staged_documents(&tx)?;
            delete_staged_attachments(&tx)?;

            tx.commit()?;
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
