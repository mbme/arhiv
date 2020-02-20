use crate::config::ArhivConfig;
use crate::entities::*;
use crate::storage::Storage;
use anyhow::*;
use reqwest::blocking::{multipart, Client};

pub struct Replica {
    storage: Storage,
    config: ArhivConfig,
}

impl Replica {
    pub fn open(config: ArhivConfig) -> Replica {
        let root_dir = &config.arhiv_root.clone();
        Replica {
            config,
            storage: Storage::open(root_dir).expect("storage must exist"),
        }
    }

    pub fn create(config: ArhivConfig) -> Result<Replica> {
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

    pub fn get_document(&self, id: &str) -> Option<Document> {
        self.storage.get_document_local(id)?;

        self.storage.get_document(id)
    }

    fn sync(&self) -> Result<()> {
        let (changeset, files) = self.storage.get_changeset();

        let mut form = multipart::Form::new().text("changeset", changeset.serialize());

        for (id, path) in files {
            form = form.file(id, path)?;
        }

        let resp = Client::new()
            .post(&self.config.primary_url)
            .multipart(form)
            .send()?;

        Ok(())
    }
}
