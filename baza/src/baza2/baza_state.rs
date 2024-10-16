use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, Write},
};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id, InstanceId, LatestRevComputer, Revision};

use super::baza_storage::BazaInfo;

#[derive(Serialize, Deserialize, Debug)]
pub enum DocumentHead {
    Document(Document),
    Conflict(Vec<Document>),

    NewDocument(Document),
    Updated {
        original: Document,
        updated: Document,
    },
    ResolvedConflict {
        original: Vec<Document>,
        updated: Document,
    },
}

impl DocumentHead {
    pub fn get_revision(&self) -> HashSet<&Revision> {
        let mut revs = HashSet::new();

        match self {
            DocumentHead::Document(original) | DocumentHead::Updated { original, .. } => {
                revs.insert(original.rev.as_ref().unwrap_or_default());
            }
            DocumentHead::Conflict(original) | DocumentHead::ResolvedConflict { original, .. } => {
                original.iter().for_each(|document| {
                    revs.insert(document.rev.as_ref().unwrap_or_default());
                });
            }
            DocumentHead::NewDocument(_) => {}
        };

        revs
    }

    pub fn is_committed(&self) -> bool {
        matches!(self, DocumentHead::Document(_) | DocumentHead::Conflict(_))
    }

    pub fn is_modified(&self) -> bool {
        !self.is_committed()
    }

    pub fn is_unresolved_conflict(&self) -> bool {
        matches!(self, DocumentHead::Conflict(_))
    }

    pub fn reset(self) -> Option<Self> {
        let result = match self {
            DocumentHead::Document(_) | DocumentHead::Conflict(_) => self,
            DocumentHead::Updated { original, .. } => DocumentHead::Document(original),
            DocumentHead::ResolvedConflict { original, .. } => DocumentHead::Conflict(original),
            DocumentHead::NewDocument(_) => return None,
        };

        Some(result)
    }

    pub fn into_modified_document(self) -> Option<Document> {
        let result = match self {
            DocumentHead::Document(_) | DocumentHead::Conflict(_) => return None,
            DocumentHead::Updated { updated, .. } => updated,
            DocumentHead::ResolvedConflict { updated, .. } => updated,
            DocumentHead::NewDocument(document) => document,
        };

        Some(result)
    }

    pub fn get_modified_document(&self) -> Option<&Document> {
        let result = match self {
            DocumentHead::Document(_) | DocumentHead::Conflict(_) => return None,
            DocumentHead::Updated { updated, .. } => updated,
            DocumentHead::ResolvedConflict { updated, .. } => updated,
            DocumentHead::NewDocument(document) => document,
        };

        Some(result)
    }

    pub fn update(self, new_document: Document) -> Self {
        match self {
            DocumentHead::Document(document) => DocumentHead::Updated {
                original: document,
                updated: new_document,
            },
            DocumentHead::Updated { original, .. } => DocumentHead::Updated {
                original,
                updated: new_document,
            },
            DocumentHead::Conflict(original) => DocumentHead::ResolvedConflict {
                original,
                updated: new_document,
            },
            DocumentHead::ResolvedConflict { original, .. } => DocumentHead::ResolvedConflict {
                original,
                updated: new_document,
            },
            DocumentHead::NewDocument(_document) => DocumentHead::NewDocument(new_document),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BazaState {
    info: BazaInfo,
    documents: HashMap<Id, DocumentHead>,
}

// TODO kvs
impl BazaState {
    pub fn new(info: BazaInfo, documents: HashMap<Id, DocumentHead>) -> Self {
        Self { info, documents }
    }

    pub fn read(reader: impl BufRead) -> Result<Self> {
        serde_json::from_reader(reader).context("Failed to parse BazaState")
    }

    pub fn write(&self, writer: impl Write) -> Result<()> {
        serde_json::to_writer(writer, &self).context("Failed to serialize BazaState")
    }

    pub fn get_info(&self) -> Result<&BazaInfo> {
        Ok(&self.info)
    }

    pub fn get_latest_revision(&self) -> HashSet<&Revision> {
        let mut latest_rev_computer = LatestRevComputer::new();

        for document in self.iter_documents() {
            let document_revs = document.get_revision();
            latest_rev_computer.update(document_revs);
        }

        latest_rev_computer.get()
    }

    fn calculate_next_revision(&self, instance_id: &InstanceId) -> Revision {
        let all_revs = self
            .iter_documents()
            .flat_map(|head| head.get_revision())
            .collect::<Vec<_>>();

        Revision::compute_next_rev(all_revs.as_slice(), instance_id)
    }

    pub fn get_document(&self, id: &Id) -> Result<&DocumentHead> {
        self.documents.get(id).context("can't find document")
    }

    pub fn put_document(&mut self, mut new_document: Document) -> Result<()> {
        let id = new_document.id.clone();

        let current_value = self.documents.remove(&id);

        new_document.stage();

        let updated_document = if let Some(document_head) = current_value {
            document_head.update(new_document)
        } else {
            DocumentHead::NewDocument(new_document)
        };

        self.documents.insert(id, updated_document);

        Ok(())
    }

    pub fn iter_documents(&self) -> impl Iterator<Item = &DocumentHead> {
        self.documents.values()
    }

    pub fn iter_modified_documents(&self) -> impl Iterator<Item = &Document> {
        self.iter_documents()
            .filter_map(|document_head| document_head.get_modified_document())
    }

    pub fn is_modified(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_modified())
    }

    pub fn has_unresolved_conflicts(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_unresolved_conflict())
    }

    pub fn reset_all_documents(&mut self) {
        let ids = self
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_modified().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        for id in ids {
            let (id, document) = self.documents.remove_entry(&id).expect("entry must exist");

            if let Some(reset_document) = document.reset() {
                self.documents.insert(id, reset_document);
            }
        }
    }

    pub fn reset_document(&mut self, id: &Id) -> Result<()> {
        let (id, document) = self
            .documents
            .remove_entry(id)
            .context("Document doesn't exist")?;

        if let Some(reset_document) = document.reset() {
            self.documents.insert(id, reset_document);
        }

        Ok(())
    }

    pub fn commit(&mut self, instance_id: &InstanceId) -> Result<Vec<Document>> {
        let ids = self
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_modified().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        ensure!(!ids.is_empty(), "Nothing to commit");

        let new_rev = self.calculate_next_revision(instance_id);

        let mut new_documents = Vec::with_capacity(ids.len());

        for id in ids {
            let (id, document_head) = self.documents.remove_entry(&id).expect("entry must exist");

            let mut document = document_head
                .into_modified_document()
                .expect("document must be modified");
            document.rev = Some(new_rev.clone());

            new_documents.push(document.clone());

            self.documents.insert(id, DocumentHead::Document(document));
        }

        Ok(new_documents)
    }
}
