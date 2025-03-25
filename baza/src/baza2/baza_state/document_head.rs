use std::{collections::HashSet, fmt};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::entities::{Document, DocumentKey, DocumentType, Id, Revision, VectorClockOrder};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DocumentHead {
    original: HashSet<Document>,
    staged: Option<Document>,
    snapshots_count: usize,
}

impl DocumentHead {
    pub fn new(document: Document) -> Self {
        if document.is_staged() {
            Self {
                original: HashSet::with_capacity(0),
                staged: Some(document),
                snapshots_count: 0,
            }
        } else {
            Self {
                original: HashSet::from_iter([document]),
                staged: None,
                snapshots_count: 1,
            }
        }
    }

    #[cfg(test)]
    pub fn new_with_snapshots_count(document: Document, snapshots_count: usize) -> Self {
        let mut doc = DocumentHead::new(document);
        doc.snapshots_count = snapshots_count;

        doc
    }

    pub fn new_conflict(revisions: impl Iterator<Item = Document>) -> Result<Self> {
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
            snapshots_count: original.len(),

            original,
            staged: None,
        })
    }

    pub fn get_id(&self) -> &Id {
        if let Some(staged) = &self.staged {
            return &staged.id;
        }

        &self
            .original
            .iter()
            .next()
            .expect("original must not be empty")
            .id
    }

    pub fn get_type(&self) -> &DocumentType {
        if let Some(staged) = &self.staged {
            return &staged.document_type;
        }

        &self
            .original
            .iter()
            .next()
            .expect("original must not be empty")
            .document_type
    }

    pub fn get_revision(&self) -> &Revision {
        if let Some(staged) = &self.staged {
            return &staged.rev;
        }

        &self
            .original
            .iter()
            .next()
            .expect("original must not be empty")
            .rev
    }

    pub fn get_original_revisions(&self) -> impl Iterator<Item = &Revision> {
        self.original.iter().map(|doc| &doc.rev)
    }

    pub fn get_snapshots_count(&self) -> usize {
        self.snapshots_count
    }

    pub fn create_key(&self) -> DocumentKey {
        DocumentKey::new(self.get_id().clone(), self.get_revision().clone())
    }

    pub fn is_committed(&self) -> bool {
        !self.is_staged()
    }

    pub fn is_staged(&self) -> bool {
        self.staged.is_some()
    }

    pub fn is_staged_erased(&self) -> bool {
        self.staged.as_ref().is_some_and(|doc| doc.is_erased())
    }

    pub fn is_original_erased(&self) -> bool {
        !self.is_conflict()
            && self
                .original
                .iter()
                .next()
                .map(|doc| doc.is_erased())
                .unwrap_or_default()
    }

    pub fn is_new_document(&self) -> bool {
        self.original.is_empty()
    }

    pub fn is_conflict(&self) -> bool {
        self.original.len() > 1
    }

    pub fn is_unresolved_conflict(&self) -> bool {
        self.is_conflict() && self.staged.is_none()
    }

    pub fn is_resolved_conflict(&self) -> bool {
        self.is_conflict() && self.staged.is_some()
    }

    pub fn reset(mut self) -> Option<Self> {
        if self.is_new_document() {
            return None;
        }

        self.staged = None;

        Some(self)
    }

    pub fn commit(self, new_rev: Revision) -> Result<Self> {
        let rev = self.get_revision();
        ensure!(
            new_rev > *rev,
            "New revision must be newer than current revision"
        );

        let mut staged_document = self.staged.context("Expected staged document")?;

        staged_document.rev = new_rev;

        Ok(DocumentHead::new(staged_document))
    }

    pub fn modify(&mut self, mut new_document: Document) -> Result<()> {
        ensure!(
            self.get_id() == &new_document.id,
            "Document id must not change"
        );

        ensure!(
            !self.is_original_erased(),
            "Erased document must not change"
        );

        new_document.stage();
        self.staged = Some(new_document);

        Ok(())
    }

    pub fn insert_snapshot(mut self, new_document: Document) -> Result<Self> {
        ensure!(
            self.get_id() == &new_document.id,
            "Document id must not change"
        );
        ensure!(!self.is_staged(), "Can't insert into staged document");

        match self.get_revision().compare_vector_clocks(&new_document.rev) {
            VectorClockOrder::Before => Ok(DocumentHead::new(new_document)),
            VectorClockOrder::Concurrent => {
                let inserted = self.original.insert(new_document);
                ensure!(inserted, "Conflict already contains this document");

                Ok(self)
            }
            VectorClockOrder::After => bail!("Can't insert document with older rev"),
            VectorClockOrder::Equal => bail!("Can't insert document with the same rev"),
        }
    }

    pub(super) fn update_snapshots_count(&mut self, snapshots_count: usize) {
        self.snapshots_count = snapshots_count;
    }

    pub fn iter_original_snapshots(&self) -> impl Iterator<Item = &Document> {
        self.original.iter()
    }

    pub fn iter_all_snapshots(&self) -> impl Iterator<Item = &Document> {
        self.staged.iter().chain(self.original.iter())
    }

    pub fn get_single_document(&self) -> &Document {
        self.iter_all_snapshots()
            .next()
            .expect("snapshots must not be empty")
    }
}

impl fmt::Display for DocumentHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.get_id();

        if self.is_new_document() {
            return write!(f, "[DocumentHead {id}, new]");
        }

        if self.is_conflict() {
            let num_revs = self.original.len();

            if self.is_resolved_conflict() {
                return write!(
                    f,
                    "[DocumentHead conflict: {num_revs} revs of {id}, staged]"
                );
            } else {
                return write!(f, "[DocumentHead conflict: {num_revs} revs of {id}]");
            }
        }

        if self.is_staged() {
            return write!(f, "[DocumentHead {id}, staged]");
        }

        write!(f, "[DocumentHead {id}]")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::entities::{new_document, Revision};

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
            assert_eq!(head.snapshots_count, 2);

            let doc_c1 = doc_a1.clone().with_rev(json!({ "c": 1 }));
            assert!(head.clone().insert_snapshot(doc_c1).unwrap().is_conflict());

            assert!(!head
                .clone()
                .insert_snapshot(doc_a3.clone())
                .unwrap()
                .is_conflict());

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_resolved_conflict());
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            assert!(!head.is_unresolved_conflict());
            assert!(head.is_committed());
            assert!(!head.is_staged());
            assert_eq!(head.snapshots_count, 1);

            assert!(head
                .clone()
                .insert_snapshot(doc_a2.clone())
                .unwrap()
                .is_conflict());

            assert!(!head
                .clone()
                .insert_snapshot(doc_a3.clone())
                .unwrap()
                .is_conflict());

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
            assert_eq!(head.snapshots_count, 0);

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
            assert_eq!(head.snapshots_count, 1);

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
            assert_eq!(head.snapshots_count, 2);

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
            assert_eq!(head.get_revision(), &new_rev);
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            head.modify(doc_a2.clone()).unwrap();
            assert!(!head.is_committed());

            head = head.commit(new_rev.clone()).unwrap();
            assert!(head.is_committed());
            assert_eq!(head.get_revision(), &new_rev);
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
            assert!(head.is_original_erased());
        }
    }

    #[test]
    fn test_iter_snapshots() {
        let doc_a1 = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "b": 1 }));
        let doc_a3 = doc_a1.clone().with_rev(json!({ "a": 1, "b": 1, "c": 1 }));

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            assert_eq!(head.iter_all_snapshots().count(), 1);

            head.modify(doc_a2.clone()).unwrap();
            assert_eq!(head.iter_all_snapshots().count(), 2);
        }

        {
            let mut head =
                DocumentHead::new_conflict([doc_a1.clone(), doc_a2.clone()].into_iter()).unwrap();
            assert_eq!(head.iter_all_snapshots().count(), 2);

            head.modify(doc_a3.clone()).unwrap();
            assert_eq!(head.iter_all_snapshots().count(), 3);
        }
    }

    #[test]
    fn test_update_snapshots_count() {
        {
            let doc_a1 = new_document(json!({})).with_rev(json!({ "a": 1 }));
            let mut head = DocumentHead::new(doc_a1.clone());
            assert_eq!(head.snapshots_count, 1);

            head.update_snapshots_count(3);

            assert_eq!(head.snapshots_count, 3);
        }

        {
            let doc_a1 = new_document(json!({}));
            let mut head = DocumentHead::new(doc_a1.clone());
            assert_eq!(head.snapshots_count, 0);

            head.update_snapshots_count(3);

            assert_eq!(head.snapshots_count, 3);
        }
    }
}
