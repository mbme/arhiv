use std::{
    collections::{HashMap, HashSet},
    fmt,
    io::{BufRead, Write},
};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    create_file_reader, create_file_writer,
    crypto_key::CryptoKey,
};

use crate::entities::{Document, Id, InstanceId, LatestRevComputer, Revision, VectorClockOrder};

use super::baza_storage::BazaInfo;

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

    pub fn reset(&mut self) {
        self.staged = None;
    }

    pub fn modify(&mut self, mut new_document: Document) -> Result<()> {
        ensure!(
            self.original.id == new_document.id,
            "Document id must not change"
        );

        new_document.stage();

        if self.is_new() {
            self.original = new_document;
        } else {
            self.staged = Some(new_document);
        }

        Ok(())
    }

    pub fn insert_revision(&mut self, new_document: Document) -> Result<()> {
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

    pub fn insert_revision(&mut self, new_document: Document) -> Result<()> {
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

    pub fn is_committed(&self) -> bool {
        match self {
            DocumentHead::Document(latest_document) => latest_document.is_committed(),
            DocumentHead::Conflict(latest_conflict) => latest_conflict.is_committed(),
        }
    }

    pub fn is_staged(&self) -> bool {
        !self.is_committed()
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

    pub fn into_staged_document(self) -> Option<Document> {
        match self {
            DocumentHead::Document(latest_document) => latest_document.into_staged_document(),
            DocumentHead::Conflict(latest_conflict) => latest_conflict.into_staged_document(),
        }
    }

    pub fn modify(&mut self, new_document: Document) -> Result<()> {
        match self {
            DocumentHead::Document(ref mut latest_document) => latest_document.modify(new_document),
            DocumentHead::Conflict(ref mut latest_conflict) => latest_conflict.modify(new_document),
        }
    }

    pub fn insert_revision(self, new_document: Document) -> Result<Self> {
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
                        latest_document.insert_revision(new_document)?;

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
                        latest_conflict.insert_revision(new_document)?;

                        DocumentHead::Conflict(latest_conflict)
                    }
                    VectorClockOrder::After => bail!("Can't insert document with older rev"),
                    VectorClockOrder::Equal => bail!("Can't insert document with the same rev"),
                }
            }
        };

        Ok(new_head)
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BazaState {
    instance_id: InstanceId,
    info: BazaInfo,
    documents: HashMap<Id, DocumentHead>,
}

// TODO kvs
impl BazaState {
    pub fn new(
        instance_id: InstanceId,
        info: BazaInfo,
        documents: HashMap<Id, DocumentHead>,
    ) -> Self {
        Self {
            info,
            documents,
            instance_id,
        }
    }

    #[cfg(test)]
    pub fn new_test_state() -> Self {
        Self::new(
            InstanceId::from_string("test"),
            BazaInfo::new_test_info(),
            HashMap::new(),
        )
    }

    pub fn read(reader: impl BufRead, key: &CryptoKey) -> Result<Self> {
        let c1_key = Confidential1Key::borrow_key(key);
        let c1_reader = Confidential1Reader::new(reader, &c1_key)?;

        serde_json::from_reader(c1_reader).context("Failed to parse BazaState")
    }

    pub fn read_file(file: &str, key: &CryptoKey) -> Result<Self> {
        let state_reader = create_file_reader(file)?;
        BazaState::read(state_reader, key)
    }

    pub fn write(&self, writer: impl Write, key: &CryptoKey) -> Result<()> {
        let c1_key = Confidential1Key::borrow_key(key);
        let mut c1_writer = Confidential1Writer::new(writer, &c1_key)?;

        serde_json::to_writer(&mut c1_writer, &self).context("Failed to serialize BazaState")?;

        c1_writer.finish()?;

        Ok(())
    }

    pub fn write_to_file(&self, file: &str, key: &CryptoKey) -> Result<()> {
        let mut state_writer = create_file_writer(file)?;

        self.write(&mut state_writer, key)?;

        state_writer.flush()?;

        Ok(())
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

    pub fn get_single_latest_revision(&self) -> &Revision {
        self.get_latest_revision()
            .into_iter()
            .next()
            .expect("revision set must not be empty")
    }

    fn calculate_next_revision(&self) -> Revision {
        let all_revs = self.iter_documents().flat_map(|head| head.get_revision());

        Revision::compute_next_rev(all_revs, &self.instance_id)
    }

    pub fn get_document(&self, id: &Id) -> Option<&DocumentHead> {
        self.documents.get(id)
    }

    pub fn modify_document(&mut self, mut document: Document) -> Result<()> {
        let id = document.id.clone();

        let current_value = self.documents.remove(&id);

        document.stage();

        let updated_document = if let Some(mut document_head) = current_value {
            document_head.modify(document)?;
            document_head
        } else {
            DocumentHead::new(document)
        };

        self.documents.insert(id, updated_document);

        Ok(())
    }

    pub fn insert_snapshot(&mut self, document: Document) -> Result<()> {
        let id = document.id.clone();

        let current_value = self.documents.remove(&id);

        let updated_document = if let Some(document_head) = current_value {
            document_head.insert_revision(document)?
        } else {
            DocumentHead::new(document)
        };

        self.documents.insert(id, updated_document);

        Ok(())
    }

    #[cfg(test)]
    pub fn insert_snapshots(&mut self, documents: Vec<Document>) {
        for document in documents {
            self.insert_snapshot(document)
                .expect("must insert document");
        }
    }

    pub fn iter_documents(&self) -> impl Iterator<Item = &DocumentHead> {
        self.documents.values()
    }

    pub fn has_staged_documents(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_staged())
    }

    pub fn has_unresolved_conflicts(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_unresolved_conflict())
    }

    pub fn reset_all_documents(&mut self) {
        let ids = self
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_staged().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        for id in &ids {
            self.reset_document(id).expect("entry must exist");
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
            .filter_map(|(id, document)| document.is_staged().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        ensure!(!ids.is_empty(), "Nothing to commit");

        let new_rev = self.calculate_next_revision();

        let mut new_documents = Vec::with_capacity(ids.len());

        for id in ids {
            let (id, document_head) = self.documents.remove_entry(&id).expect("entry must exist");

            let mut document = document_head
                .into_staged_document()
                .expect("document must be modified");
            document.rev = new_rev.clone();

            new_documents.push(document.clone());

            self.documents.insert(id, DocumentHead::new(document));
        }

        Ok(new_documents)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rs_utils::crypto_key::CryptoKey;
    use serde_json::json;

    use crate::{entities::Revision, tests::new_document};

    use super::{BazaState, DocumentHead};

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
                head.clone().insert_revision(doc_c1).unwrap(),
                DocumentHead::Conflict(_)
            ));

            assert!(matches!(
                head.clone().insert_revision(doc_a3.clone()).unwrap(),
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
                head.clone().insert_revision(doc_a2.clone()).unwrap(),
                DocumentHead::Conflict(_)
            ));

            assert!(matches!(
                head.clone().insert_revision(doc_a3.clone()).unwrap(),
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
            assert!(head.clone().insert_revision(doc_a3.clone()).is_err());

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_new_document());
        }

        {
            let mut head = DocumentHead::new(doc_a1.clone());
            head.modify(doc_a2.clone()).unwrap();

            assert!(!head.is_unresolved_conflict());
            assert!(!head.is_committed());
            assert!(head.is_staged());
            assert!(head.clone().insert_revision(doc_a3.clone()).is_err());

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
            assert!(head.clone().insert_revision(doc_a3.clone()).is_err());

            head.modify(doc_a3.clone()).unwrap();
            assert!(head.is_resolved_conflict());
        }
    }

    #[test]
    fn test_state() {
        let mut state = BazaState::new_test_state();

        assert_eq!(state.get_single_latest_revision(), Revision::INITIAL);

        let doc_a1 = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "test": 1 }));
        let mut doc_a3 = doc_a1.clone();
        doc_a3.stage();

        state.insert_snapshots(vec![doc_a1, doc_a2]);
        assert!(!state.has_staged_documents());
        assert!(state.has_unresolved_conflicts());
        assert_eq!(state.get_latest_revision().len(), 2);

        state.modify_document(doc_a3.clone()).unwrap();
        assert!(state.has_staged_documents());
        assert!(!state.has_unresolved_conflicts());

        state.reset_all_documents();
        assert!(!state.has_staged_documents());
        assert!(state.has_unresolved_conflicts());

        state.modify_document(doc_a3).unwrap();
        let new_documents = state.commit().unwrap();

        let new_rev = Revision::from_value(json!({ "a": 1, "test": 2 })).unwrap();
        assert_eq!(new_documents.len(), 1);
        assert_eq!(new_documents[0].rev, new_rev);
        assert!(!state.has_staged_documents());
        assert!(!state.has_unresolved_conflicts());
        assert_eq!(state.get_single_latest_revision(), &new_rev);
    }

    #[test]
    fn test_state_read_write() {
        let key = CryptoKey::new_random_key();
        let mut state = BazaState::new_test_state();

        state.insert_snapshots(vec![
            new_document(json!({ "test": 1 })).with_rev(json!({ "a": 1 })),
            new_document(json!({ "test": 2 })).with_rev(json!({ "a": 2, "b": 2 })),
        ]);

        let mut data = Cursor::new(Vec::<u8>::new());

        state.write(&mut data, &key).unwrap();
        data.set_position(0);

        let state1 = BazaState::read(&mut data, &key).unwrap();

        assert_eq!(state, state1);
    }
}
