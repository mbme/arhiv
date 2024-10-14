use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id, LatestRevComputer, Revision};

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
    pub fn is_committed(&self) -> bool {
        matches!(self, DocumentHead::Document(_) | DocumentHead::Conflict(_))
    }

    pub fn get_revision(&self) -> Result<HashSet<&Revision>> {
        let mut revs = HashSet::new();

        match self {
            DocumentHead::Document(original) | DocumentHead::Updated { original, .. } => {
                revs.insert(original.get_rev()?);
            }
            DocumentHead::Conflict(original) | DocumentHead::ResolvedConflict { original, .. } => {
                let original_revs = original
                    .iter()
                    .map(|document| document.get_rev())
                    .collect::<Result<Vec<_>>>()?;

                revs.extend(original_revs);
            }
            DocumentHead::NewDocument(_) => {}
        };

        Ok(revs)
    }

    pub fn could_reset(&self) -> bool {
        matches!(
            self,
            DocumentHead::Updated { .. } | DocumentHead::ResolvedConflict { .. }
        )
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
    pub fn new(info: BazaInfo) -> Self {
        Self {
            info,
            documents: HashMap::new(),
        }
    }

    pub fn get_info(&self) -> Result<&BazaInfo> {
        Ok(&self.info)
    }

    pub fn get_latest_revision(&self) -> Result<HashSet<&Revision>> {
        let mut latest_rev_computer = LatestRevComputer::new();

        for document in self.iter_documents()? {
            let document_revs = document.get_revision()?;
            latest_rev_computer.update(document_revs)?;
        }

        Ok(latest_rev_computer.get())
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

    pub fn iter_documents(&self) -> Result<impl Iterator<Item = &DocumentHead>> {
        Ok(self.documents.values())
    }

    pub fn iter_uncommitted_documents(&self) -> Result<impl Iterator<Item = &DocumentHead>> {
        Ok(self
            .iter_documents()?
            .filter(|document| !document.is_committed()))
    }

    pub fn reset_all_documents(&mut self) {
        let ids = self
            .documents
            .iter()
            .filter_map(|(id, document)| document.could_reset().then_some(id))
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
}
