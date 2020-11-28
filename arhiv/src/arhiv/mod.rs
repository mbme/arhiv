use crate::config::Config;
use crate::entities::*;
use crate::storage::*;
use anyhow::*;
use chrono::Utc;
use rs_utils::FsTransaction;
use serde::{Deserialize, Serialize};
pub use server::start_server;
use std::sync::Arc;

mod server;
mod sync;

pub struct Arhiv {
    storage: Storage,
    config: Arc<Config>,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open(Config::must_read()).expect("must be able to open arhiv")
    }

    pub fn open(config: Config) -> Result<Arhiv> {
        let config = Arc::new(config);
        let storage = Storage::open(config.clone())?;

        Ok(Arhiv { config, storage })
    }

    pub fn create(prime: bool, config: Config) -> Result<Arhiv> {
        let config = Arc::new(config);
        let storage = Storage::create(prime, config.clone())?;

        Ok(Arhiv { config, storage })
    }

    pub fn get_root_dir(&self) -> &str {
        &self.config.arhiv_root
    }

    pub fn get_status(&self) -> Result<Status> {
        let conn = self.storage.get_connection()?;

        let root_dir = self.get_root_dir().to_string();
        let rev = conn.get_rev()?;
        let (committed_documents, staged_documents) = conn.count_documents()?;
        let (committed_attachments, staged_attachments) = conn.count_attachments()?;
        let is_prime = conn.is_prime()?;

        Ok(Status {
            root_dir,
            rev,
            is_prime,
            committed_documents,
            staged_documents,
            committed_attachments,
            staged_attachments,
        })
    }

    pub fn list_documents(&self, filter: Option<DocumentFilter>) -> Result<ListPage<Document>> {
        let conn = self.storage.get_connection()?;

        conn.list_documents(filter.unwrap_or_default())
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.storage.get_connection()?;

        conn.get_document(id)
    }

    pub fn stage_document(
        &self,
        updated_document: Document,
        new_attachments: Vec<AttachmentSource>,
    ) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;

        let mut fs_tx = FsTransaction::new();

        let mut document = {
            if let Some(mut document) = conn.get_document(&updated_document.id)? {
                document.rev = Revision::STAGING; // make sure document rev is Staging
                document.updated_at = Utc::now();
                document.data = updated_document.data;

                document
            } else {
                let mut new_document = Document::new(updated_document.data);
                new_document.id = updated_document.id;

                new_document
            }
        };

        document.archived = updated_document.archived;
        document.refs = updated_document.refs;

        // Validate document references
        let new_attachments_ids: Vec<&Id> = new_attachments.iter().map(|item| &item.id).collect();
        for reference in document.refs.iter() {
            // FIXME optimize validating id
            if conn.get_document(reference)?.is_some() {
                continue;
            }
            if conn.get_attachment(reference)?.is_some() {
                continue;
            }
            if reference == &document.id {
                log::warn!("Document {} references itself", &document.id);
                continue;
            }
            if new_attachments_ids.contains(&reference) {
                continue;
            }

            bail!(
                "Document {} reference unknown entity {}",
                &document.id,
                reference
            );
        }

        // Stage new attachments
        for new_attachment in new_attachments {
            if !document.refs.contains(&new_attachment.id) {
                log::warn!(
                    "Document {} new attachment is unused, ignoring: {}",
                    &document.id,
                    &new_attachment
                );
                continue;
            }

            if conn.get_attachment(&new_attachment.id)?.is_some() {
                log::warn!(
                    "Document {} new attachment already exists, ignoring: {}",
                    &document.id,
                    &new_attachment
                );
                continue;
            }

            let attachment = Attachment::from(&new_attachment)?;
            conn.put_attachment(&attachment, false)?;

            let path = self
                .storage
                .get_attachment_data(attachment.id.clone())
                .get_staged_file_path();

            if new_attachment.copy {
                fs_tx.copy_file(new_attachment.file_path.to_string(), path)?;
            } else {
                fs_tx.hard_link_file(new_attachment.file_path.to_string(), path)?;
            }

            log::debug!(
                "staged new attachment {}: {}",
                attachment,
                new_attachment.file_path
            );
        }

        conn.put_document(&document)?;

        conn.commit()?;
        fs_tx.commit();

        // FIXME remove unused staged attachments

        log::trace!("staged document {}", &document);

        Ok(())
    }

    pub fn list_attachments(
        &self,
        filter: Option<AttachmentFilter>,
    ) -> Result<ListPage<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.list_attachments(filter.unwrap_or_default())
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.get_attachment(id)
    }

    pub fn update_attachment_filename<S: Into<String>>(&self, id: &Id, filename: S) -> Result<()> {
        let mut attachment = self
            .get_attachment(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;
        attachment.filename = filename.into();

        if attachment.rev.is_staged() {
            let mut conn = self.storage.get_writable_connection()?;
            let tx = conn.get_tx()?;

            tx.put_attachment(&attachment, true)?;

            tx.commit()?;

            return Ok(());
        }

        if self.storage.get_connection()?.is_prime()? {
            let mut conn = self.storage.get_writable_connection()?;
            let tx = conn.get_tx()?;

            let current_rev = tx.get_rev()?;

            attachment.rev = current_rev.inc();

            tx.put_attachment(&attachment, true)?;

            tx.commit()?;

            return Ok(());
        }

        bail!("committed attachment filename must be updated on Prime");
    }

    pub fn get_attachment_data(&self, id: &Id) -> AttachmentData {
        self.storage.get_attachment_data(id.clone())
    }

    pub fn get_attachment_location(&self, id: &Id) -> Result<AttachmentLocation> {
        let attachment = self
            .get_attachment(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;

        let data = self.storage.get_attachment_data(id.clone());

        if attachment.rev.is_staged() {
            if !data.staged_file_exists()? {
                bail!("can't find staged file for attachment {}", id);
            }

            return Ok(AttachmentLocation::File(data.get_staged_file_path()));
        }

        if data.committed_file_exists()? {
            return Ok(AttachmentLocation::File(data.get_committed_file_path()));
        }

        Ok(AttachmentLocation::Url(data.get_url()?))
    }
}

#[derive(Serialize, Deserialize)]
pub enum AttachmentLocation {
    Url(String),
    File(String),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub root_dir: String,
    pub is_prime: bool,
    pub rev: Revision,

    pub committed_documents: u32,
    pub staged_documents: u32,

    pub committed_attachments: u32,
    pub staged_attachments: u32,
}

#[cfg(test)]
impl Drop for Arhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.get_root_dir()).expect("must be able to remove arhiv");
    }
}
