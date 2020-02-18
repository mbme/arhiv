use crate::entities::*;
use crate::storage::Storage;

pub struct Replica {
    storage: Storage,
}

impl Replica {
    pub fn open(path: &str) -> Replica {
        Replica {
            storage: Storage::open(path).expect("storage must exist"),
        }
    }

    pub fn create(path: &str, primary_url: &str) -> Replica {
        Replica {
            storage: Storage::create(path, primary_url).expect("must be able to create storage"),
        }
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

    fn sync(&self) {}
}
