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

    pub fn get_single_document(&self) -> &Document {
        match self {
            DocumentHead::Document(document) => document,
            DocumentHead::Conflict(documents) => &documents[0],
            DocumentHead::NewDocument(document) => document,
            DocumentHead::Updated { updated, .. } => updated,
            DocumentHead::ResolvedConflict { updated, .. } => updated,
        }
    }

    pub fn modify(self, new_document: Document) -> Result<Self> {
        let result = match self {
            DocumentHead::Document(document) => {
                ensure!(
                    document.id == new_document.id,
                    "Document id must not change"
                );

                DocumentHead::Updated {
                    original: document,
                    updated: new_document,
                }
            }
            DocumentHead::Updated { original, .. } => {
                ensure!(
                    original.id == new_document.id,
                    "Document id must not change"
                );

                DocumentHead::Updated {
                    original,
                    updated: new_document,
                }
            }
            DocumentHead::Conflict(original) => {
                ensure!(
                    original[0].id == new_document.id,
                    "Document id must not change"
                );

                DocumentHead::ResolvedConflict {
                    original,
                    updated: new_document,
                }
            }
            DocumentHead::ResolvedConflict { original, .. } => {
                ensure!(
                    original[0].id == new_document.id,
                    "Document id must not change"
                );

                DocumentHead::ResolvedConflict {
                    original,
                    updated: new_document,
                }
            }
            DocumentHead::NewDocument(document) => {
                ensure!(
                    document.id == new_document.id,
                    "Document id must not change"
                );

                DocumentHead::NewDocument(new_document)
            }
        };

        Ok(result)
    }

    pub fn insert_revision(self, new_document: Document) -> Result<Self> {
        let new_rev = new_document.get_rev()?;

        let new_head = match self {
            DocumentHead::Document(document) => {
                ensure!(
                    document.id == new_document.id,
                    "Document id must not change"
                );

                let rev = document.get_rev()?;

                match rev.compare_vector_clocks(new_rev) {
                    VectorClockOrder::Before => DocumentHead::Document(new_document),
                    VectorClockOrder::Concurrent => {
                        DocumentHead::Conflict(vec![document, new_document])
                    }
                    VectorClockOrder::After => bail!("Can't insert document with older rev"),
                    VectorClockOrder::Equal => bail!("Can't insert document with the same rev"),
                }
            }
            DocumentHead::Conflict(mut original) => {
                let document = &original[0];
                ensure!(
                    document.id == new_document.id,
                    "Document id must not change"
                );

                let rev = document.get_rev()?;

                match rev.compare_vector_clocks(new_rev) {
                    VectorClockOrder::Before => DocumentHead::Document(new_document),
                    VectorClockOrder::Concurrent => {
                        let has_rev = original.iter().any(|document| {
                            document.rev.as_ref().is_some_and(|rev| rev == new_rev)
                        });
                        ensure!(!has_rev, "Spcified document rev already exists");

                        original.push(new_document);
                        DocumentHead::Conflict(original)
                    }
                    VectorClockOrder::After => bail!("Can't insert document with older rev"),
                    VectorClockOrder::Equal => bail!("Can't insert document with the same rev"),
                }
            }
            _ => bail!("Can't insert into modified document"),
        };

        Ok(new_head)
    }
}

impl fmt::Display for DocumentHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentHead::Document(document) => {
                write!(f, "[DocumentHead/Document {document}]")
            }
            DocumentHead::Conflict(documents) => {
                write!(
                    f,
                    "[DocumentHead/Conflict {} revs of {}]",
                    documents.len(),
                    documents[0].id
                )
            }
            DocumentHead::NewDocument(document) => {
                write!(f, "[DocumentHead/NewDocument {document}]")
            }
            DocumentHead::Updated { original, .. } => {
                write!(f, "[DocumentHead/Updated {original}]")
            }
            DocumentHead::ResolvedConflict { original, .. } => {
                write!(
                    f,
                    "[DocumentHead/ResolvedConflict {} revs of {}]",
                    original.len(),
                    original[0].id
                )
            }
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

        let updated_document = if let Some(document_head) = current_value {
            document_head.modify(document)?
        } else {
            DocumentHead::NewDocument(document)
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
            DocumentHead::Document(document)
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
            let head = DocumentHead::Conflict(vec![doc_a1.clone(), doc_a2.clone()]);
            assert!(head.is_unresolved_conflict());
            assert!(head.is_committed());
            assert!(!head.is_modified());

            let doc_c1 = doc_a1.clone().with_rev(json!({ "c": 1 }));
            assert!(matches!(
                head.clone().insert_revision(doc_c1).unwrap(),
                DocumentHead::Conflict(_)
            ));

            assert!(matches!(
                head.clone().insert_revision(doc_a3.clone()).unwrap(),
                DocumentHead::Document(_)
            ));

            assert!(matches!(
                head.clone().modify(doc_a3.clone()).unwrap(),
                DocumentHead::ResolvedConflict { .. }
            ));
        }

        {
            let head = DocumentHead::Document(doc_a1.clone());
            assert!(!head.is_unresolved_conflict());
            assert!(head.is_committed());
            assert!(!head.is_modified());

            assert!(matches!(
                head.clone().insert_revision(doc_a2.clone()).unwrap(),
                DocumentHead::Conflict(_)
            ));

            assert!(matches!(
                head.clone().insert_revision(doc_a3.clone()).unwrap(),
                DocumentHead::Document(_)
            ));

            assert!(matches!(
                head.clone().modify(doc_a3.clone()).unwrap(),
                DocumentHead::Updated { .. }
            ));
        }

        {
            let head = DocumentHead::NewDocument(doc_a1.clone());
            assert!(!head.is_unresolved_conflict());
            assert!(!head.is_committed());
            assert!(head.is_modified());
            assert!(head.clone().insert_revision(doc_a3.clone()).is_err());

            assert!(matches!(
                head.clone().modify(doc_a3.clone()).unwrap(),
                DocumentHead::NewDocument(_)
            ));
        }

        {
            let head = DocumentHead::Updated {
                original: doc_a1.clone(),
                updated: doc_a2.clone(),
            };
            assert!(!head.is_unresolved_conflict());
            assert!(!head.is_committed());
            assert!(head.is_modified());
            assert!(head.clone().insert_revision(doc_a3.clone()).is_err());

            assert!(matches!(
                head.clone().modify(doc_a3.clone()).unwrap(),
                DocumentHead::Updated { .. }
            ));
        }

        {
            let head = DocumentHead::ResolvedConflict {
                original: vec![doc_a1.clone(), doc_a2.clone()],
                updated: doc_a3.clone(),
            };
            assert!(!head.is_unresolved_conflict());
            assert!(!head.is_committed());
            assert!(head.is_modified());
            assert!(head.clone().insert_revision(doc_a3.clone()).is_err());

            assert!(matches!(
                head.clone().modify(doc_a3.clone()).unwrap(),
                DocumentHead::ResolvedConflict { .. }
            ));
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
        assert!(!state.is_modified());
        assert!(state.has_unresolved_conflicts());
        assert_eq!(state.get_latest_revision().len(), 2);

        state.modify_document(doc_a3.clone()).unwrap();
        assert!(state.is_modified());
        assert!(!state.has_unresolved_conflicts());

        state.reset_all_documents();
        assert!(!state.is_modified());
        assert!(state.has_unresolved_conflicts());

        state.modify_document(doc_a3).unwrap();
        let new_documents = state.commit().unwrap();

        let new_rev = Revision::from_value(json!({ "a": 1, "test": 2 })).unwrap();
        assert_eq!(new_documents.len(), 1);
        assert_eq!(*new_documents[0].get_rev().unwrap(), new_rev);
        assert!(!state.is_modified());
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
