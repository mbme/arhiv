use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, Write},
};

use anyhow::{bail, ensure, Context, Result};
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

    pub fn modify(self, new_document: Document) -> Self {
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

    pub fn insert(self, new_document: Document) -> Result<Self> {
        let new_rev = new_document.get_rev()?;

        let new_head = match self {
            DocumentHead::Document(document) => {
                let rev = document.get_rev()?;

                if rev.is_concurrent(new_rev) {
                    DocumentHead::Conflict(vec![document, new_document])
                } else {
                    DocumentHead::Document(new_document)
                }
            }
            DocumentHead::Conflict(mut original) => {
                let rev = original.first().context("must not be empty")?.get_rev()?;

                if rev.is_concurrent(new_rev) {
                    original.push(new_document);
                    DocumentHead::Conflict(original)
                } else {
                    DocumentHead::Document(new_document)
                }
            }
            _ => bail!("Can't insert into modified document"),
        };

        Ok(new_head)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BazaState {
    instance_id: InstanceId,
    info: BazaInfo,
    documents: HashMap<Id, DocumentHead>,
}

// TODO kvs
impl BazaState {
    pub fn new(info: BazaInfo, documents: HashMap<Id, DocumentHead>) -> Self {
        let instance_id = InstanceId::generate();

        Self {
            info,
            documents,
            instance_id,
        }
    }

    pub fn read(reader: impl BufRead) -> Result<Self> {
        serde_json::from_reader(reader).context("Failed to parse BazaState")
    }

    pub fn write(&self, writer: impl Write) -> Result<()> {
        serde_json::to_writer(writer, &self).context("Failed to serialize BazaState")
    }

    pub fn get_info(&self) -> &BazaInfo {
        &self.info
    }

    pub fn get_latest_revision(&self) -> HashSet<&Revision> {
        let mut latest_rev_computer = LatestRevComputer::new();

        for document in self.iter_documents() {
            let document_revs = document.get_revision();
            latest_rev_computer.update(document_revs);
        }

        latest_rev_computer.get()
    }

    fn calculate_next_revision(&self) -> Revision {
        let all_revs = self
            .iter_documents()
            .flat_map(|head| head.get_revision())
            .collect::<Vec<_>>();

        Revision::compute_next_rev(all_revs.as_slice(), &self.instance_id)
    }

    pub fn get_document(&self, id: &Id) -> Option<&DocumentHead> {
        self.documents.get(id)
    }

    pub fn modify_document(&mut self, mut document: Document) -> Result<()> {
        let id = document.id.clone();

        let current_value = self.documents.remove(&id);

        document.stage();

        let updated_document = if let Some(document_head) = current_value {
            document_head.modify(document)
        } else {
            DocumentHead::NewDocument(document)
        };

        self.documents.insert(id, updated_document);

        Ok(())
    }

    pub(super) fn insert_document(&mut self, document: Document) -> Result<()> {
        let id = document.id.clone();

        let current_value = self.documents.remove(&id);

        let updated_document = if let Some(document_head) = current_value {
            document_head.insert(document)?
        } else {
            DocumentHead::Document(document)
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

    pub fn commit(&mut self) -> Result<Vec<Document>> {
        let ids = self
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_modified().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        ensure!(!ids.is_empty(), "Nothing to commit");

        let new_rev = self.calculate_next_revision();

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
