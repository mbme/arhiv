use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, Write},
};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    create_file_reader, create_file_writer,
    crypto_key::CryptoKey,
};

use crate::{
    baza2::BazaInfo,
    entities::{Document, Id, InstanceId, LatestRevComputer, Revision},
};

mod document_head;

pub use document_head::{DocumentHead, LatestConflict, LatestDocument};

// FIXME separate on-disk data structure from in-memory data structure
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BazaState {
    instance_id: InstanceId,
    info: BazaInfo,
    documents: HashMap<Id, DocumentHead>,
}

// TODO kvs
// TODO locks
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

    pub fn read(reader: impl BufRead, key: CryptoKey) -> Result<Self> {
        let c1_key = Confidential1Key::new(key);
        let c1_reader = Confidential1Reader::new(reader, &c1_key)?;

        serde_json::from_reader(c1_reader).context("Failed to parse BazaState")
    }

    pub fn read_file(file: &str, key: CryptoKey) -> Result<Self> {
        let state_reader = create_file_reader(file)?;
        BazaState::read(state_reader, key)
    }

    pub fn write(&self, writer: impl Write, key: CryptoKey) -> Result<()> {
        let c1_key = Confidential1Key::new(key);
        let mut c1_writer = Confidential1Writer::new(writer, &c1_key)?;

        serde_json::to_writer(&mut c1_writer, &self).context("Failed to serialize BazaState")?;

        c1_writer.finish()?;

        Ok(())
    }

    pub fn write_to_file(&self, file: &str, key: CryptoKey) -> Result<()> {
        let mut state_writer = create_file_writer(file, true)?;

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

    pub fn stage_document(&mut self, mut document: Document) -> Result<()> {
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

    pub(super) fn insert_snapshot(&mut self, document: Document) -> Result<()> {
        let id = document.id.clone();

        let current_value = self.documents.remove(&id);

        let updated_document = if let Some(document_head) = current_value {
            document_head.insert_snapshot(document)?
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

    pub fn iter_snapshots(&self) -> impl Iterator<Item = &Document> {
        self.iter_documents().flat_map(|head| head.iter_snapshots())
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

    pub(super) fn commit(&mut self) -> Result<()> {
        let ids = self
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_staged().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        ensure!(!ids.is_empty(), "Nothing to commit");

        let new_rev = self.calculate_next_revision();

        for id in ids {
            let (id, document_head) = self.documents.remove_entry(&id).expect("entry must exist");

            let new_head = document_head.commit(new_rev.clone())?;

            self.documents.insert(id, new_head);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rs_utils::crypto_key::CryptoKey;
    use serde_json::json;

    use crate::{entities::Revision, tests::new_document};

    use super::BazaState;

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

        state.stage_document(doc_a3.clone()).unwrap();
        assert!(state.has_staged_documents());
        assert!(!state.has_unresolved_conflicts());

        state.reset_all_documents();
        assert!(!state.has_staged_documents());
        assert!(state.has_unresolved_conflicts());

        state.stage_document(doc_a3.clone()).unwrap();
        state.commit().unwrap();

        let new_rev = Revision::from_value(json!({ "a": 1, "test": 2 })).unwrap();

        assert!(state.get_document(&doc_a3.id).unwrap().is_committed());
        assert_eq!(
            state
                .get_document(&doc_a3.id)
                .unwrap()
                .get_single_latest_revision(),
            &new_rev
        );

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

        state.write(&mut data, key.clone()).unwrap();
        data.set_position(0);

        let state1 = BazaState::read(&mut data, key.clone()).unwrap();

        assert_eq!(state, state1);
    }
}
