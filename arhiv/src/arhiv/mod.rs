use crate::config::Config;
use crate::entities::*;
use crate::fs_transaction::FsTransaction;
use crate::storage::*;
use crate::utils::{ensure_file_exists, file_exists};
use anyhow::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

mod server;
mod sync;

#[derive(Serialize, Deserialize)]
pub enum AttachmentLocation {
    Url(String),
    File(String),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub is_prime: bool,
    pub rev: u32,

    pub commited_documents: u32,
    pub staged_documents: u32,

    pub commited_attachments: u32,
    pub staged_attachments: u32,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(self).expect("Failed to serialize status to json")
        )
    }
}

pub struct Arhiv {
    storage: Storage,
    config: Config,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open(Config::must_read()).expect("must be able to open arhiv")
    }

    pub fn open(config: Config) -> Result<Arhiv> {
        let root_dir = &config.arhiv_root.clone();

        Ok(Arhiv {
            config,
            storage: Storage::open(root_dir)?,
        })
    }

    pub fn create(config: Config) -> Result<()> {
        let root_dir = &config.arhiv_root.clone();

        Storage::create(root_dir)?;

        Ok(())
    }

    pub fn get_status(&self) -> Result<Status> {
        let conn = self.storage.get_connection()?;

        let rev = conn.get_rev()?;
        let (commited_documents, staged_documents) = conn.count_documents()?;
        let (commited_attachments, staged_attachments) = conn.count_attachments()?;

        Ok(Status {
            rev,
            is_prime: self.config.prime,
            commited_documents,
            staged_documents,
            commited_attachments,
            staged_attachments,
        })
    }

    pub fn list_documents(&self, filter: Option<QueryFilter>) -> Result<Vec<Document>> {
        let conn = self.storage.get_connection()?;

        conn.get_documents(0, filter.unwrap_or_default())
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.storage.get_connection()?;

        conn.get_document(id)
    }

    pub fn stage_document(&self, mut updated_document: Document) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;

        if let Some(mut document) = conn.get_document(&updated_document.id)? {
            document.rev = 0; // make sure document rev is Staging
            document.updated_at = Utc::now();
            document.data = updated_document.data;
            document.refs = updated_document.refs;
            document.attachment_refs = updated_document.attachment_refs;

            conn.put_document(&document)?;
            conn.commit()?;
            log::trace!("staged document {}", &document);
        } else {
            updated_document.rev = 0;
            updated_document.created_at = Utc::now();
            updated_document.updated_at = Utc::now();

            conn.put_document(&updated_document)?;
            conn.commit()?;
            log::trace!("staged new document {}", &updated_document);
        }

        Ok(())
    }

    pub fn list_attachments(&self) -> Result<Vec<Attachment>> {
        // FIXME pagination
        let conn = self.storage.get_connection()?;

        if self.config.prime {
            conn.get_committed_attachments()
        } else {
            conn.get_all_attachments()
        }
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let conn = self.storage.get_connection()?;

        conn.get_attachment(id)
    }

    pub fn get_attachment_location(&self, id: &Id) -> Result<AttachmentLocation> {
        let attachment = self
            .get_attachment(id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;

        if attachment.is_staged() {
            let local_file_path = self.storage.get_staged_attachment_file_path(id);

            if file_exists(&local_file_path)? {
                return Ok(AttachmentLocation::File(local_file_path));
            } else {
                return Ok(AttachmentLocation::Unknown);
            }
        }

        let local_file_path = self.storage.get_committed_attachment_file_path(id);
        if file_exists(&local_file_path)? {
            return Ok(AttachmentLocation::File(local_file_path));
        }

        let primary_url = self
            .config
            .primary_url
            .as_ref()
            .ok_or(anyhow!("config.primary_url is missing"))?;

        let url = AttachmentLocation::Url(format!("{}/attachment-data/{}", primary_url, id));

        Ok(url)
    }

    pub fn stage_attachment(&self, file: &str) -> Result<Attachment> {
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
        fs_tx.hard_link_file(
            file.to_string(),
            self.storage.get_staged_attachment_file_path(&attachment.id),
        )?;

        conn.commit()?;
        fs_tx.commit();

        log::debug!("staged new attachment {}: {}", attachment, file);

        Ok(attachment)
    }
}
