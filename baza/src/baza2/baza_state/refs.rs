use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::{
    entities::{BLOBId, Document, DocumentKey, Id, Refs},
    DocumentExpert,
};

use super::{BazaState, DocumentHead};

pub type BazaRefsState = HashMap<DocumentKey, Refs>;

impl BazaState {
    pub(super) fn update_document_refs(&mut self, head: &DocumentHead) -> Result<()> {
        self.remove_document_refs(head.get_id());

        for snapshot in head.iter_all_snapshots() {
            let key = DocumentKey::for_document(snapshot);
            let snapshot_refs = self.extract_document_refs(snapshot)?;

            self.file.refs.insert(key, snapshot_refs);
        }

        Ok(())
    }

    pub(super) fn remove_document_refs(&mut self, id: &Id) {
        self.file.refs.retain(|key, _| key.id != *id);
    }

    fn extract_document_refs(&self, document: &Document) -> Result<Refs> {
        let expert = DocumentExpert::new(&self.schema);

        expert.extract_refs(&document.document_type, &document.data)
    }

    pub(super) fn update_all_documents_refs(&mut self) -> Result<()> {
        self.file.refs = self
            .iter_documents()
            .flat_map(|head| head.iter_all_snapshots())
            .map(|document| {
                let key = DocumentKey::for_document(document);
                let snapshot_refs = self.extract_document_refs(document)?;

                Ok((key, snapshot_refs))
            })
            .collect::<Result<BazaRefsState>>()?;

        Ok(())
    }

    pub fn get_document_snapshot_refs(&self, key: &DocumentKey) -> Option<&Refs> {
        self.file.refs.get(key)
    }

    pub fn get_document_refs(&self, id: &Id) -> Option<&Refs> {
        let document = self.get_document(id)?.get_single_document();

        let key = document.create_key();

        self.get_document_snapshot_refs(&key)
    }

    pub fn get_all_blob_refs(&self) -> HashSet<BLOBId> {
        let mut blob_refs = HashSet::new();

        for refs in self.file.refs.values() {
            blob_refs.extend(refs.blobs.iter().cloned());
        }

        blob_refs
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{baza2::BazaState, tests::new_empty_document};

    #[test]
    fn test_extracts_refs_on_insert() {
        let mut state = BazaState::new_test_state();
        let doc1 = new_empty_document().with_rev(json!({ "a": 1 }));
        let doc2 = new_empty_document()
            .with_rev(json!({ "a": 1 }))
            .with_data(json!({"ref": doc1.id}));

        state.insert_snapshot(doc1.clone()).unwrap();
        assert!(state.get_document_refs(&doc1.id).unwrap().is_empty());

        state.insert_snapshot(doc2.clone()).unwrap();
        assert!(!state.get_document_refs(&doc2.id).unwrap().is_empty());
    }

    #[test]
    fn test_extracts_refs_on_stage() {
        let mut state = BazaState::new_test_state();
        let doc1 = new_empty_document();
        let doc2 = new_empty_document().with_data(json!({"ref": doc1.id}));

        state.stage_document(doc1.clone(), &None).unwrap();
        assert!(state.get_document_refs(&doc1.id).unwrap().is_empty());

        state.stage_document(doc2.clone(), &None).unwrap();
        assert!(!state.get_document_refs(&doc2.id).unwrap().is_empty());
    }

    #[test]
    fn test_updates_refs_on_reset() {
        let mut state = BazaState::new_test_state();
        let doc1 = new_empty_document().with_rev(json!({ "a": 1 }));
        let doc2 = new_empty_document()
            .with_rev(json!({ "a": 1 }))
            .with_data(json!({"ref": doc1.id}));
        let doc2_1 = new_empty_document()
            .with_id(doc2.id.clone())
            .with_data(json!({}));

        state.insert_snapshot(doc1.clone()).unwrap();
        state.insert_snapshot(doc2.clone()).unwrap();

        state.stage_document(doc2_1.clone(), &None).unwrap();
        assert!(state.get_document_refs(&doc2.id).unwrap().is_empty());

        state.reset_document(&doc2.id, &None).unwrap();
        assert!(!state.get_document_refs(&doc2.id).unwrap().is_empty());
    }

    #[test]
    fn test_updates_refs_on_commit() {
        let mut state = BazaState::new_test_state();

        let doc1 = new_empty_document().with_rev(json!({ "a": 1 }));
        let doc2 = new_empty_document().with_rev(json!({ "a": 1 }));
        let doc3 = new_empty_document()
            .with_rev(json!({ "a": 1 }))
            .with_data(json!({"ref": doc1.id}));
        let doc3_1 = new_empty_document()
            .with_id(doc3.id.clone())
            .with_data(json!({"ref": doc2.id}));

        state.insert_snapshot(doc1.clone()).unwrap();
        state.insert_snapshot(doc2.clone()).unwrap();
        state.insert_snapshot(doc3.clone()).unwrap();

        state.stage_document(doc3_1.clone(), &None).unwrap();
        assert!(state
            .get_document_snapshot_refs(&doc3.create_key())
            .is_some());
        assert!(state
            .get_document_snapshot_refs(&doc3_1.create_key())
            .is_some());

        state.commit().unwrap();
        assert!(state
            .get_document_snapshot_refs(&doc3.create_key())
            .is_none());
        assert!(state
            .get_document_refs(&doc3.id)
            .is_some_and(|refs| !refs.is_empty()));
    }
}
