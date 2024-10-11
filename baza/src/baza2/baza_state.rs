use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id, LatestRevComputer, Revision};

use super::baza_storage::BazaInfo;

#[derive(Serialize, Deserialize, Debug)]
pub enum LatestDocument {
    Document(Document),
    Updated {
        original: Document,
        updated: Document,
    },
    Conflict(Vec<Document>),
    ResolvedConflict {
        original: Vec<Document>,
        updated: Document,
    },
}

impl LatestDocument {
    pub fn is_committed(&self) -> bool {
        matches!(
            self,
            LatestDocument::Document(_) | LatestDocument::Conflict(_)
        )
    }

    pub fn get_revision(&self) -> Result<HashSet<&Revision>> {
        let mut revs = HashSet::new();

        match self {
            LatestDocument::Document(original) | LatestDocument::Updated { original, .. } => {
                revs.insert(original.get_rev()?);
            }
            LatestDocument::Conflict(original)
            | LatestDocument::ResolvedConflict { original, .. } => {
                let original_revs = original
                    .iter()
                    .map(|document| document.get_rev())
                    .collect::<Result<Vec<_>>>()?;

                revs.extend(original_revs);
            }
        };

        Ok(revs)
    }

    pub fn could_reset(&self) -> bool {
        matches!(
            self,
            LatestDocument::Updated { .. } | LatestDocument::ResolvedConflict { .. }
        )
    }

    pub fn reset(self) -> Self {
        match self {
            LatestDocument::Document(_) | LatestDocument::Conflict(_) => self,
            LatestDocument::Updated { original, .. } => LatestDocument::Document(original),
            LatestDocument::ResolvedConflict { original, .. } => LatestDocument::Conflict(original),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BazaState {
    info: BazaInfo,
    documents: HashMap<Id, LatestDocument>,
}

// TODO kvs
impl BazaState {
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

    pub fn get_document(&self, id: &Id) -> Result<&LatestDocument> {
        self.documents.get(id).context("can't find document")
    }

    pub fn iter_documents(&self) -> Result<impl Iterator<Item = &LatestDocument>> {
        Ok(self.documents.values())
    }

    pub fn iter_uncommitted_documents(&self) -> Result<impl Iterator<Item = &LatestDocument>> {
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
            self.documents.insert(id, document.reset());
        }
    }

    pub fn reset_document(&mut self, id: &Id) -> Result<()> {
        let (id, document) = self
            .documents
            .remove_entry(id)
            .context("Document doesn't exist")?;

        self.documents.insert(id, document.reset());

        Ok(())
    }
}
