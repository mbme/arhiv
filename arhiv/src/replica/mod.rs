use crate::entities::*;
use crate::utils::ensure_exists;
use anyhow::*;
use chrono::Utc;
pub use config::ReplicaConfig;
use reqwest::blocking::{multipart, Client};
use std::collections::HashMap;
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
        self.storage.get_documents()
    }

    pub fn get_document(&self, id: &Id) -> Option<Document> {
        self.storage.get_document(id)
    }

    pub fn save_document(&self, mut document: Document) {
        document.rev = 0;
        document.updated_at = Utc::now();

        self.storage
            .put_document(&document)
            .expect("must be able to save local document");
    }

    pub fn get_attachments(&self) -> Vec<Attachment> {
        self.storage.get_attachments()
    }

    pub fn get_attachment(&self, id: &Id) -> Option<Attachment> {
        self.storage.get_attachment(id)
    }

    pub fn save_attachment(&self, file: &str, move_file: bool) -> Attachment {
        ensure_exists(file, false).expect("new attachment file must exist");

        let attachment = Attachment::new();

        self.storage
            .put_attachment(&attachment)
            .expect("must be able to save local attachment");
        self.storage
            .put_attachment_data(&attachment.id, file, move_file)
            .expect("must be able to save local attachment data");

        attachment
    }

    fn get_documents_local(&self) -> Vec<Document> {
        self.storage
            .get_documents()
            .into_iter()
            .filter(|item| item.rev > 0)
            .collect()
    }

    fn get_attachments_local(&self) -> Vec<Attachment> {
        self.storage
            .get_attachments()
            .into_iter()
            .filter(|item| item.rev > 0)
            .collect()
    }

    pub fn sync(&self) -> Result<()> {
        let rev = self.storage.get_rev();

        let changeset = Changeset {
            replica_rev: rev,
            documents: self.get_documents_local(),
            attachments: self.get_attachments_local(),
        };

        let mut files = HashMap::new();

        for attachment in changeset.attachments.iter() {
            files.insert(
                attachment.id.clone(),
                self.storage.get_attachment_data_path(&attachment.id),
            );
        }

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

        if resp.replica_rev != rev {
            return Err(anyhow!("replica_rev isn't equal to current rev"));
        }

        for document in resp.documents {
            self.storage.put_document(&document)?;
        }

        for attachment in resp.attachments {
            self.storage.put_attachment(&attachment)?;
            self.storage.remove_attachment_data(&attachment.id)?;
        }

        self.storage.set_rev(resp.primary_rev);

        Ok(())
    }
}
