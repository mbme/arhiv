use crate::config::Config;
use crate::entities::*;
use crate::fs_transaction::FsTransaction;
use crate::storage::*;
use crate::utils::ensure_file_exists;
use anyhow::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use status::Status;
use std::path::Path;
use std::sync::Arc;

pub mod notes;
mod server;
mod status;
mod sync;

#[cfg(test)]
mod arhiv_tests;

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

    pub fn create(config: Config) -> Result<Arhiv> {
        let config = Arc::new(config);
        let storage = Storage::create(config.clone())?;

        Ok(Arhiv { config, storage })
    }

    pub fn get_root_dir(&self) -> &str {
        &self.config.arhiv_root
    }

    pub fn get_status(&self) -> Result<Status> {
        let conn = self.storage.get_connection()?;

        let rev = conn.get_rev()?;
        let (commited_documents, staged_documents) = conn.count_documents()?;
        let (commited_attachments, staged_attachments) = conn.count_attachments()?;

        Ok(Status {
            rev,
            is_prime: self.config.is_prime,
            commited_documents,
            staged_documents,
            commited_attachments,
            staged_attachments,
        })
    }

    pub fn list_documents(&self, filter: Option<DocumentFilter>) -> Result<Vec<Document>> {
        let conn = self.storage.get_connection()?;

        conn.get_documents(0, filter.unwrap_or_default())
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.storage.get_connection()?;

        conn.get_document(id)
    }

    pub fn stage_document(&self, mut document: Document) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;

        if let Some(mut existing_document) = conn.get_document(&document.id)? {
            existing_document.rev = 0; // make sure document rev is Staging
            existing_document.updated_at = Utc::now();
            existing_document.archived = document.archived;
            existing_document.data = document.data;
            existing_document.refs = document.refs;
            existing_document.attachment_refs = document.attachment_refs;

            conn.put_document(&existing_document)?;
            conn.commit()?;
            log::trace!("staged document {}", &existing_document);
        } else {
            document.rev = 0;
            document.created_at = Utc::now();
            document.updated_at = Utc::now();

            conn.put_document(&document)?;
            conn.commit()?;
            log::trace!("staged new document {}", &document);
        }

        Ok(())
    }

    pub fn list_attachments(&self, filter: Option<AttachmentFilter>) -> Result<Vec<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.get_attachments(0, filter.unwrap_or_default())
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.get_attachment(id)
    }

    pub fn stage_attachment(&self, file: &str, copy: bool) -> Result<Attachment> {
        ensure_file_exists(file).expect("new attachment file must exist");

        let attachment = Attachment::new(
            Path::new(file)
                .file_name()
                .expect("file must have name")
                .to_str()
                .unwrap(),
        );

        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;
        let mut fs_tx = FsTransaction::new();

        conn.put_attachment(&attachment)?;

        let path = self
            .storage
            .get_attachment_data(attachment.id.clone())
            .get_staged_file_path();

        if copy {
            fs_tx.copy_file(file.to_string(), path)?;
        } else {
            fs_tx.hard_link_file(file.to_string(), path)?;
        }

        conn.commit()?;
        fs_tx.commit();

        log::debug!("staged new attachment {}: {}", attachment, file);

        Ok(attachment)
    }

    pub fn get_attachment_data(&self, id: &Id) -> AttachmentData {
        self.storage.get_attachment_data(id.clone())
    }

    pub fn get_attachment_location(&self, id: Id) -> Result<AttachmentLocation> {
        let attachment = self
            .get_attachment(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;

        let data = self.storage.get_attachment_data(id.clone());

        if attachment.is_staged() {
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
