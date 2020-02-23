use crate::entities::*;
use crate::utils::ensure_exists;
use anyhow::*;
use chrono::Utc;
pub use config::ReplicaConfig;
use reqwest::blocking::{multipart, Client};
use storage::Storage;

mod config;
mod storage;

pub struct Replica {
    storage: Storage,
    config: ReplicaConfig,
}

impl Replica {
    pub fn open(config: ReplicaConfig) -> Replica {
        let root_dir = &config.arhiv_root.clone();
        Replica {
            config,
            storage: Storage::open(root_dir).expect("storage must exist"),
        }
    }

    pub fn create(config: ReplicaConfig) -> Result<Replica> {
        let root_dir = &config.arhiv_root.clone();
        Ok(Replica {
            config,
            storage: Storage::create(root_dir)?,
        })
    }

    pub fn get_documents(&self) -> Vec<Document> {
        let mut documents = self.storage.get_documents_local();

        documents.append(&mut self.storage.get_documents());

        documents.dedup_by_key(|document| document.id.clone());

        documents
    }

    pub fn get_document(&self, id: &Id) -> Option<Document> {
        self.storage.get_document_local(id)?;

        self.storage.get_document(id)
    }

    pub fn save_document(&self, mut document: Document) {
        document.rev = 0;
        document.updated_at = Utc::now();

        self.storage
            .put_document_local(&document)
            .expect("must be able to save local document");
    }

    pub fn get_attachments(&self) -> Vec<Attachment> {
        let mut attachments = self.storage.get_attachments_local();

        attachments.append(&mut self.storage.get_attachments());

        attachments.dedup_by_key(|attachment| attachment.id.clone());

        attachments
    }

    pub fn get_attachment(&self, id: &Id) -> Option<Attachment> {
        self.storage.get_attachment_local(id)?;

        self.storage.get_attachment(id)
    }

    pub fn save_attachment(&self, file: &str, move_file: bool) -> Attachment {
        ensure_exists(file, false).expect("new attachment file must exist");

        let attachment = Attachment::new();

        self.storage
            .put_attachment_local(&attachment)
            .expect("must be able to save local attachment");
        self.storage
            .put_attachment_data(&attachment.id, file, move_file)
            .expect("must be able to save local attachment data");

        attachment
    }

    pub fn sync(&self) -> Result<()> {
        let (changeset, files) = self.storage.get_changeset();

        let mut form = multipart::Form::new().text("changeset", changeset.serialize());

        for (id, path) in files {
            form = form.file(id, path)?;
        }

        let resp: ChangesetResponse = Client::new()
            .post(&self.config.primary_url)
            .multipart(form)
            .send()?
            .text()?
            .parse()?;

        self.storage.apply_changeset_response(resp)
    }
}
