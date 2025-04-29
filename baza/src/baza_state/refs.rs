use std::collections::HashSet;

use anyhow::Result;

use crate::{
    entities::{Document, DocumentKey, Id, Refs},
    DocumentExpert,
};

use super::{BazaState, DocumentHead};

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

    pub fn get_document_snapshot_refs(&self, key: &DocumentKey) -> Option<&Refs> {
        self.file.refs.get(key)
    }

    pub fn get_document_refs(&self, id: &Id) -> Option<&Refs> {
        let document = self.get_document(id)?.get_single_document();

        let key = document.create_key();

        self.get_document_snapshot_refs(&key)
    }

    pub fn find_document_backrefs(&self, id: &Id) -> HashSet<Id> {
        let mut backrefs = HashSet::new();

        let keys = self
            .iter_documents()
            .map(|head| head.get_single_document().create_key());

        for key in keys {
            let refs = self
                .get_document_snapshot_refs(&key)
                .expect("Document refs must be known");

            if refs.documents.contains(id) {
                backrefs.insert(key.id.clone());
            }
        }

        backrefs
    }

    pub fn find_document_collections(&self, id: &Id) -> HashSet<Id> {
        let mut collections = HashSet::new();

        let keys = self
            .iter_documents()
            .map(|head| head.get_single_document().create_key());

        for key in keys {
            let refs = self
                .get_document_snapshot_refs(&key)
                .expect("Document refs must be known");

            if refs.collection.contains(id) {
                collections.insert(key.id);
            }
        }

        collections
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{entities::new_empty_document, BazaState};

    #[test]
    fn test_extracts_refs_on_insert() {
        let mut state = BazaState::new_test_state();
        let doc1 = new_empty_document().with_rev(json!({ "a": 1 }));
        let doc2 = new_empty_document()
            .with_rev(json!({ "a": 1 }))
            .with_data(json!({"ref": doc1.id}));

        state.insert_snapshots(vec![doc1.clone(), doc2.clone()]);
        assert!(state.get_document_refs(&doc1.id).unwrap().is_empty());
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

        state.insert_snapshots(vec![doc1.clone(), doc2.clone()]);

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

        state.insert_snapshots(vec![doc1.clone(), doc2.clone(), doc3.clone()]);

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
