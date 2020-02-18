use crate::entities::*;
use crate::storage::Storage;

pub struct Replica {
    storage: Storage,
}

impl Replica {
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
}
