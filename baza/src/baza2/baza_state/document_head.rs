use std::{collections::HashSet, fmt};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id, Revision, VectorClockOrder};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LatestDocument {
    original: Document,
    staged: Option<Document>,
}

impl LatestDocument {
    pub fn new(original: Document) -> Self {
        Self {
            original,
            staged: None,
        }
    }

    pub fn get_id(&self) -> &Id {
        &self.original.id
    }

    pub fn get_revision(&self) -> &Revision {
        &self.original.rev
    }

    pub fn is_new(&self) -> bool {
        self.original.is_staged()
    }

    pub fn is_old(&self) -> bool {
        !self.is_new()
    }

    pub fn is_staged(&self) -> bool {
        self.staged.is_some() || self.is_new()
    }

    pub fn is_committed(&self) -> bool {
        !self.is_staged()
    }

    pub fn is_erased(&self) -> bool {
        self.original.is_erased()
    }

    pub fn reset(&mut self) {
        self.staged = None;
    }

    pub fn modify(&mut self, mut new_document: Document) -> Result<()> {
        ensure!(
            self.original.id == new_document.id,
            "Document id must not change"
        );

        ensure!(!self.is_erased(), "Erased document must not change");

        new_document.stage();

        if self.is_new() {
            self.original = new_document;
        } else {
            self.staged = Some(new_document);
        }

        Ok(())
    }

    pub fn insert_snapshot(&mut self, new_document: Document) -> Result<()> {
        ensure!(
            self.get_id() == &new_document.id,
            "Document id must not change"
        );
        ensure!(!self.is_staged(), "Can't insert into staged document");

        self.original = new_document;

        Ok(())
    }

    pub fn into_staged_document(self) -> Option<Document> {
        if self.is_new() {
            return Some(self.original);
        }

        self.staged
    }
}

impl fmt::Display for LatestDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.get_id();

        if self.is_new() {
            write!(f, "[LatestDocument {id}, new staged]")
        } else if self.is_staged() {
            write!(f, "[LatestDocument {id}, staged]")
        } else {
            write!(f, "[LatestDocument {id}]")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LatestConflict {
    original: HashSet<Document>,
    staged: Option<Document>,
}

impl LatestConflict {
    pub fn new(revisions: impl Iterator<Item = Document>) -> Result<Self> {
        let original = HashSet::from_iter(revisions);
        ensure!(
            original.len() > 1,
            "Conflict must contain at least two revisions"
        );

        let id = &original.iter().next().expect("original can't be empty").id;

        let all_ids_are_equal = original.iter().all(|doc| doc.id == *id);
        ensure!(
            all_ids_are_equal,
            "All conflict revisions must have the same id"
        );

        Ok(Self {
            original,
            staged: None,
        })
    }

    pub fn get_id(&self) -> &Id {
        &self.original.iter().next().expect("must not be empty").id
    }

    pub fn get_single_revision(&self) -> &Revision {
        &self.original.iter().next().expect("must not be empty").rev
    }

    pub fn get_revisions(&self) -> HashSet<&Revision> {
        self.original.iter().map(|doc| &doc.rev).collect()
    }

    pub fn is_staged(&self) -> bool {
        self.staged.is_some()
    }

    pub fn is_committed(&self) -> bool {
        !self.is_staged()
    }

    pub fn is_unresolved(&self) -> bool {
        !self.is_staged()
    }

    pub fn reset(&mut self) {
        self.staged = None;
    }

    pub fn modify(&mut self, mut new_document: Document) -> Result<()> {
        ensure!(
            self.get_id() == &new_document.id,
            "Document id must not change"
        );

        new_document.stage();

        self.staged = Some(new_document);

        Ok(())
    }

    pub fn insert_snapshot(&mut self, new_document: Document) -> Result<()> {
        ensure!(
            self.get_id() == &new_document.id,
            "Document id must not change"
        );
        ensure!(!self.is_staged(), "Can't insert into staged document");

        let inserted = self.original.insert(new_document);
        ensure!(inserted, "Conflict already contains this document");

        Ok(())
    }

    pub fn into_staged_document(self) -> Option<Document> {
        self.staged
    }
}

impl fmt::Display for LatestConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.get_id();
        let num_revs = self.original.len();

        if self.is_staged() {
            write!(f, "[LatestConflict {num_revs} revs of {id}, staged]")
        } else {
            write!(f, "[LatestConflict {num_revs} revs of {id}]")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum DocumentHead {
    Document(LatestDocument),
    Conflict(LatestConflict),
}

impl DocumentHead {
    pub fn new(document: Document) -> Self {
        DocumentHead::Document(LatestDocument::new(document))
    }

    pub fn new_conflict(revisions: impl Iterator<Item = Document>) -> Result<Self> {
        Ok(DocumentHead::Conflict(LatestConflict::new(revisions)?))
    }

    pub fn get_id(&self) -> &Id {
        match self {
            DocumentHead::Document(latest_document) => latest_document.get_id(),
            DocumentHead::Conflict(latest_conflict) => latest_conflict.get_id(),
        }
    }

    pub fn get_revision(&self) -> HashSet<&Revision> {
        match self {
            DocumentHead::Document(latest_document) => {
                HashSet::from([latest_document.get_revision()])
            }
            DocumentHead::Conflict(latest_conflict) => latest_conflict.get_revisions(),
        }
    }

    pub fn get_single_revision(&self) -> &Revision {
        self.get_revision()
            .into_iter()
            .next()
            .expect("revision set must not be empty")
    }

    pub fn is_committed(&self) -> bool {
        match self {
            DocumentHead::Document(latest_document) => latest_document.is_committed(),
            DocumentHead::Conflict(latest_conflict) => latest_conflict.is_committed(),
        }
    }

    pub fn is_staged(&self) -> bool {
        !self.is_committed()
    }

    pub fn is_erased(&self) -> bool {
        matches!(&self, DocumentHead::Document(latest_document) if latest_document.is_erased())
    }

    pub fn is_new_document(&self) -> bool {
        matches!(self, DocumentHead::Document(latest_document) if latest_document.is_new())
    }

    pub fn is_old_document(&self) -> bool {
        matches!(self, DocumentHead::Document(latest_document) if latest_document.is_old())
    }

    pub fn is_unresolved_conflict(&self) -> bool {
        matches!(self, DocumentHead::Conflict(latest_conflict) if latest_conflict.is_unresolved())
    }

    pub fn is_resolved_conflict(&self) -> bool {
        matches!(self, DocumentHead::Conflict(latest_conflict) if !latest_conflict.is_unresolved())
    }

    pub fn reset(mut self) -> Option<Self> {
        match self {
            DocumentHead::Document(ref mut latest_document) => {
                if latest_document.is_new() {
                    return None;
                }

                if latest_document.is_staged() {
                    latest_document.reset();
                }

                Some(self)
            }
            DocumentHead::Conflict(ref mut latest_conflict) => {
                if latest_conflict.is_staged() {
                    latest_conflict.reset();
                }

                Some(self)
            }
        }
    }

    pub fn commit(self, new_rev: Revision) -> Result<Self> {
        let staged_document = match self {
            DocumentHead::Document(latest_document) => {
                let rev = latest_document.get_revision();
                ensure!(
                    new_rev > *rev,
                    "New revision must be newer than current revision"
                );

                latest_document.into_staged_document()
            }
            DocumentHead::Conflict(latest_conflict) => {
                let rev = latest_conflict.get_single_revision();
                ensure!(
                    new_rev > *rev,
                    "New revision must be newer than current revision"
                );

                latest_conflict.into_staged_document()
            }
        };

        let mut staged_document = staged_document.context("Expected staged document")?;

        staged_document.rev = new_rev;

        Ok(DocumentHead::new(staged_document))
    }

    pub fn modify(&mut self, new_document: Document) -> Result<()> {
        match self {
            DocumentHead::Document(ref mut latest_document) => latest_document.modify(new_document),
            DocumentHead::Conflict(ref mut latest_conflict) => latest_conflict.modify(new_document),
        }
    }

    pub fn insert_snapshot(self, new_document: Document) -> Result<Self> {
        ensure!(
            self.get_id() == &new_document.id,
            "Document id must not change"
        );
        ensure!(!self.is_staged(), "Can't insert into staged document");

        let new_head = match self {
            DocumentHead::Document(mut latest_document) => {
                match latest_document
                    .get_revision()
                    .compare_vector_clocks(&new_document.rev)
                {
                    VectorClockOrder::Before => {
                        latest_document.insert_snapshot(new_document)?;

                        DocumentHead::Document(latest_document)
                    }
                    VectorClockOrder::Concurrent => {
                        let conflict = LatestConflict::new(
                            [latest_document.original, new_document].into_iter(),
                        );

                        DocumentHead::Conflict(conflict?)
                    }
                    VectorClockOrder::After => bail!("Can't insert document with older rev"),
                    VectorClockOrder::Equal => bail!("Can't insert document with the same rev"),
                }
            }
            DocumentHead::Conflict(mut latest_conflict) => {
                match latest_conflict
                    .get_single_revision()
                    .compare_vector_clocks(&new_document.rev)
                {
                    VectorClockOrder::Before => {
                        DocumentHead::Document(LatestDocument::new(new_document))
                    }
                    VectorClockOrder::Concurrent => {
                        latest_conflict.insert_snapshot(new_document)?;

                        DocumentHead::Conflict(latest_conflict)
                    }
                    VectorClockOrder::After => bail!("Can't insert document with older rev"),
                    VectorClockOrder::Equal => bail!("Can't insert document with the same rev"),
                }
            }
        };

        Ok(new_head)
    }

    pub fn iter_snapshots<'i>(&'i self) -> Box<dyn Iterator<Item = &'i Document> + 'i> {
        match self {
            DocumentHead::Document(latest_document) => {
                Box::new([&latest_document.original].into_iter())
            }
            DocumentHead::Conflict(latest_conflict) => Box::new(latest_conflict.original.iter()),
        }
    }

    pub fn get_single_snapshot(&self) -> &Document {
        self.iter_snapshots()
            .next()
            .expect("snapshots must not be empty")
    }
}

impl fmt::Display for DocumentHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentHead::Document(latest_document) => latest_document.fmt(f),
            DocumentHead::Conflict(latest_conflict) => latest_conflict.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{entities::Revision, tests::new_document};

    use super::DocumentHead;

    #[test]
    fn test_document_head() {
        let doc_a1 = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "b": 1 }));
        let doc_a3 = doc_a1.clone().with_rev(json!({ "a": 1, "b": 1, "c": 1 }));

        {
            let mut head =
                DocumentHead::new_conflict([doc_a1.clone(), doc_a2.clone()].into_iter()).unwrap();
            assert!(head.is_unresolved_conflict());
            assert!(head.is_committed());
            assert!(!head.is_staged());

            let doc_c1 = doc_a1.clone().with_rev(json!({ "c": 1 }));
            assert!(matches!(
                head.clone().insert_snapshot(doc_c1).unwrap(),
                DocumentHead::Conflict(_)
            ));

            assert!(matches!(
                head.clone().insert_snapshot(doc_a3.clone()).unwrap(),
                DocumentHead::Document(_)
            ));

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_resolved_conflict());
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            assert!(!head.is_unresolved_conflict());
            assert!(head.is_committed());
            assert!(!head.is_staged());

            assert!(matches!(
                head.clone().insert_snapshot(doc_a2.clone()).unwrap(),
                DocumentHead::Conflict(_)
            ));

            assert!(matches!(
                head.clone().insert_snapshot(doc_a3.clone()).unwrap(),
                DocumentHead::Document(_)
            ));

            head.modify(doc_a3.clone()).unwrap();

            assert!(head.is_staged());
        }

        {
            let mut doc = doc_a1.clone();
            doc.stage();
            let mut head = DocumentHead::new(doc);
            assert!(head.is_new_document());
            assert!(!head.is_committed());
            assert!(head.is_staged());
            assert!(head.clone().insert_snapshot(doc_a3.clone()).is_err());

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_new_document());
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            head.modify(doc_a2.clone()).unwrap();

            assert!(!head.is_unresolved_conflict());
            assert!(!head.is_committed());
            assert!(head.is_staged());
            assert!(head.clone().insert_snapshot(doc_a3.clone()).is_err());

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_staged());
        }

        {
            let mut head =
                DocumentHead::new_conflict([doc_a1.clone(), doc_a2.clone()].into_iter()).unwrap();
            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_resolved_conflict());
            assert!(!head.is_committed());
            assert!(head.is_staged());
            assert!(head.clone().insert_snapshot(doc_a3.clone()).is_err());

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_resolved_conflict());
        }
    }

    #[test]
    fn test_commit() {
        let doc_a1 = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "b": 1 }));
        let doc_a3 = doc_a1.clone().with_rev(json!({ "a": 1, "b": 1, "c": 1 }));

        let new_rev = Revision::from_value(json!({ "a": 2, "b": 1, "c": 1 })).unwrap();

        {
            let mut head =
                DocumentHead::new_conflict([doc_a1.clone(), doc_a2.clone()].into_iter()).unwrap();

            assert!(head.clone().commit(new_rev.clone()).is_err());

            head.modify(doc_a3.clone()).unwrap();
            assert!(!head.is_committed());

            head = head.commit(new_rev.clone()).unwrap();
            assert!(head.is_committed());
            assert_eq!(head.get_single_revision(), &new_rev);
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            head.modify(doc_a2.clone()).unwrap();
            assert!(!head.is_committed());

            head = head.commit(new_rev.clone()).unwrap();
            assert!(head.is_committed());
            assert_eq!(head.get_single_revision(), &new_rev);
        }

        {
            let mut doc_a1 = doc_a1.clone();
            doc_a1.erase();

            let mut head = DocumentHead::new(doc_a1);
            assert!(head.modify(doc_a2.clone()).is_err());
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());

            let mut doc_a2 = doc_a2.clone();
            doc_a2.erase();
            head.modify(doc_a2).unwrap();

            head = head.commit(new_rev.clone()).unwrap();
            assert!(head.is_committed());
            assert!(head.is_erased());
        }
    }
}
