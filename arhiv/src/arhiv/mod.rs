use crate::config::Config;
use crate::entities::*;
use crate::storage::*;
use anyhow::*;
use chrono::Utc;
use rs_utils::{ensure_file_exists, get_file_hash_sha256, FsTransaction};
use serde::{Deserialize, Serialize};
pub use server::start_server;
use std::path::Path;
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

    pub fn list_documents(&self, filter: Option<DocumentFilter>) -> Result<Vec<Document>> {
        let conn = self.storage.get_connection()?;

        conn.list_documents(filter.unwrap_or_default())
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

            conn.put_document(&existing_document, true)?;
            conn.commit()?;
            log::trace!("staged document {}", &existing_document);
        } else {
            document.rev = 0;
            document.created_at = Utc::now();
            document.updated_at = Utc::now();

            conn.put_document(&document, true)?;
            conn.commit()?;
            log::trace!("staged new document {}", &document);
        }

        Ok(())
    }

    pub fn list_attachments(&self, filter: Option<AttachmentFilter>) -> Result<Vec<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.list_attachments(filter.unwrap_or_default())
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.get_attachment(id)
    }

    pub fn stage_attachment(&self, file: &str, copy: bool) -> Result<Attachment> {
        ensure_file_exists(file).expect("new attachment file must exist");

        let attachment = Attachment::new(
            get_file_hash_sha256(file)?,
            Path::new(file)
                .file_name()
                .expect("file must have name")
                .to_str()
                .expect("file name must be valid string"),
        );

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.get_tx()?;
        let mut fs_tx = FsTransaction::new();

        tx.put_attachment(&attachment)?;

        let path = self
            .storage
            .get_attachment_data(attachment.id.clone())
            .get_staged_file_path();

        if copy {
            fs_tx.copy_file(file.to_string(), path)?;
        } else {
            fs_tx.hard_link_file(file.to_string(), path)?;
        }

        tx.commit()?;
        fs_tx.commit();

        log::debug!("staged new attachment {}: {}", attachment, file);

        Ok(attachment)
    }

    pub fn update_attachment_filename<S: Into<String>>(&self, id: &Id, filename: S) -> Result<()> {
        let mut attachment = self
            .get_attachment(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;
        attachment.filename = filename.into();

        if attachment.is_staged() {
            let mut conn = self.storage.get_writable_connection()?;
            let tx = conn.get_tx()?;

            tx.put_attachment(&attachment)?;

            tx.commit()?;

            return Ok(());
        }

        if self.storage.get_connection()?.is_prime()? {
            let mut conn = self.storage.get_writable_connection()?;
            let tx = conn.get_tx()?;

            let current_rev = tx.get_rev()?;

            attachment.rev = current_rev + 1;

            tx.put_attachment(&attachment)?;

            tx.commit()?;

            return Ok(());
        }

        bail!("committed attachment filename must be updated on Prime");
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub root_dir: String,
    pub is_prime: bool,
    pub rev: u32,

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
