use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, Write},
    time::Instant,
};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{
    age::AgeKey, create_file_reader, create_file_writer, log, AgeGzReader, AgeGzWriter, Timestamp,
};

use crate::{
    baza2::BazaInfo,
    entities::{Document, DocumentLockKey, Id, InstanceId, LatestRevComputer, Revision},
    schema::DataSchema,
    DocumentExpert,
};

mod document_head;
mod locks;
mod query;
mod refs;

pub use document_head::DocumentHead;
pub use locks::Locks;
pub use query::{Filter, ListPage};
use refs::BazaRefsState;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct BazaStateFile {
    instance_id: InstanceId,
    info: BazaInfo,
    documents: HashMap<Id, DocumentHead>,
    locks: Locks,
    refs: BazaRefsState,
}

pub struct BazaState {
    file: BazaStateFile,
    schema: DataSchema,
    modified: bool,
}

impl BazaState {
    pub fn new(
        instance_id: InstanceId,
        info: BazaInfo,
        schema: DataSchema,
        documents: HashMap<Id, DocumentHead>,
    ) -> Result<Self> {
        let is_empty = documents.is_empty();

        let mut baza_state = Self {
            file: BazaStateFile {
                info,
                documents,
                locks: HashMap::new(),
                refs: HashMap::new(),
                instance_id,
            },
            schema,
            modified: !is_empty,
        };

        baza_state.update_all_documents_refs()?;

        Ok(baza_state)
    }

    #[cfg(test)]
    pub fn new_test_state() -> Self {
        Self::new(
            InstanceId::from_string("test"),
            BazaInfo::new_test_info(),
            DataSchema::new_test_schema(),
            HashMap::new(),
        )
        .expect("must create test state")
    }

    pub fn read(reader: impl BufRead, key: AgeKey, schema: DataSchema) -> Result<Self> {
        let agegz_reader = AgeGzReader::new(reader, key)?;

        let file =
            serde_json::from_reader(agegz_reader).context("Failed to parse BazaStateFile")?;

        Ok(Self {
            file,
            schema,
            modified: false,
        })
    }

    pub fn read_file(file: &str, key: AgeKey, schema: DataSchema) -> Result<Self> {
        log::debug!("Reading state from file {file}");

        let start_time = Instant::now();

        let state_reader = create_file_reader(file)?;
        let state = BazaState::read(state_reader, key, schema)?;

        let duration = start_time.elapsed();
        log::info!("Read state from file in {:?}", duration);

        Ok(state)
    }

    pub fn write(&mut self, writer: impl Write, key: AgeKey) -> Result<()> {
        let mut agegz_writer = AgeGzWriter::new(writer, key)?;

        serde_json::to_writer(&mut agegz_writer, &self.file)
            .context("Failed to serialize BazaStateFile")?;

        agegz_writer.finish()?;

        self.modified = false;

        Ok(())
    }

    pub fn write_to_file(&mut self, file: &str, key: AgeKey) -> Result<()> {
        log::debug!("Writing state to file {file}");

        let start_time = Instant::now();

        let mut state_writer = create_file_writer(file, true)?;

        self.write(&mut state_writer, key)?;

        state_writer.flush()?;

        let duration = start_time.elapsed();
        log::info!("Wrote state to file in {:?}", duration);

        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn get_info(&self) -> &BazaInfo {
        &self.file.info
    }

    pub fn get_instance_id(&self) -> &InstanceId {
        &self.file.instance_id
    }

    pub fn get_schema(&self) -> &DataSchema {
        &self.schema
    }

    pub fn find_last_modification_time(&self) -> Option<Timestamp> {
        self.iter_documents()
            .map(|head| head.get_single_document().updated_at)
            .max()
    }

    pub fn get_latest_revision(&self) -> HashSet<&Revision> {
        let mut latest_rev_computer = LatestRevComputer::new();

        for document in self.iter_documents() {
            let document_revs = document.get_original_revisions();
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
        let all_revs = self
            .iter_documents()
            .flat_map(|head| head.get_original_revisions());

        Revision::compute_next_rev(all_revs, &self.file.instance_id)
    }

    pub fn get_mut_document(&mut self, id: &Id) -> Option<&mut DocumentHead> {
        self.file.documents.get_mut(id)
    }

    pub fn get_document(&self, id: &Id) -> Option<&DocumentHead> {
        self.file.documents.get(id)
    }

    pub fn must_get_document(&self, id: &Id) -> Result<&Document> {
        let head = self.get_document(id).context("Can't find document")?;

        Ok(head.get_single_document())
    }

    pub fn stage_document(
        &mut self,
        mut document: Document,
        lock_key: &Option<DocumentLockKey>,
    ) -> Result<&Document> {
        let id = document.id.clone();

        self.check_document_lock(&id, lock_key)?;

        let current_value = self.file.documents.remove(&id);

        document.stage();

        let updated_head = if let Some(mut document_head) = current_value {
            document_head.modify(document)?;
            document_head
        } else {
            DocumentHead::new(document)
        };

        self.update_document_refs(&updated_head)?;
        self.file.documents.insert(id.clone(), updated_head);
        self.modified = true;

        let document = self
            .get_document(&id)
            .context("Document must exist")?
            .get_single_document();

        Ok(document)
    }

    pub(super) fn insert_snapshot(&mut self, document: Document) -> Result<()> {
        ensure!(!document.is_staged(), "Can't insert staged document");

        let id = document.id.clone();

        let current_value = self.file.documents.remove(&id);

        let updated_head = if let Some(document_head) = current_value {
            document_head.insert_snapshot(document)?
        } else {
            DocumentHead::new(document)
        };

        self.update_document_refs(&updated_head)?;
        self.file.documents.insert(id, updated_head);
        self.modified = true;

        Ok(())
    }

    #[cfg(test)]
    pub fn insert_snapshots(&mut self, documents: Vec<Document>) {
        for document in documents {
            self.insert_snapshot(document)
                .expect("must insert document");
        }
    }

    pub(super) fn update_snapshots_count(&mut self, id: &Id, snapshots_count: usize) -> Result<()> {
        let head = self.get_mut_document(id).expect("must find document");

        if head.get_snapshots_count() != snapshots_count {
            head.update_snapshots_count(snapshots_count);
            self.modified = true;
        }

        Ok(())
    }

    pub fn iter_documents(&self) -> impl Iterator<Item = &DocumentHead> {
        self.file.documents.values()
    }

    pub fn has_staged_documents(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_staged())
    }

    pub fn has_unresolved_conflicts(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_unresolved_conflict())
    }

    pub fn reset_all_documents(&mut self) -> Result<()> {
        let ids = self
            .file
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_staged().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        for id in &ids {
            self.reset_document(id, &None)?;
        }

        Ok(())
    }

    pub fn reset_document(&mut self, id: &Id, lock_key: &Option<DocumentLockKey>) -> Result<()> {
        self.check_document_lock(id, lock_key)?;

        let (id, document) = self
            .file
            .documents
            .remove_entry(id)
            .context("Document doesn't exist")?;
        self.remove_document_refs(&id);

        if let Some(updated_head) = document.reset() {
            self.update_document_refs(&updated_head)?;
            self.file.documents.insert(id, updated_head);
        }

        self.modified = true;

        Ok(())
    }

    pub(super) fn commit(&mut self) -> Result<()> {
        ensure!(!self.has_document_locks(), "Some documents are locked");

        let ids = self
            .file
            .documents
            .iter()
            .filter_map(|(id, document)| document.is_staged().then_some(id))
            .cloned()
            .collect::<Vec<_>>();

        ensure!(!ids.is_empty(), "Nothing to commit");

        let new_rev = self.calculate_next_revision();

        for id in ids {
            let (id, document_head) = self
                .file
                .documents
                .remove_entry(&id)
                .expect("entry must exist");

            let updated_head = document_head.commit(new_rev.clone())?;

            self.update_document_refs(&updated_head)?;
            self.file.documents.insert(id, updated_head);
        }

        self.modified = true;

        Ok(())
    }

    pub fn update_document_collections(
        &mut self,
        document_id: &Id,
        collections: &Vec<Id>,
    ) -> Result<()> {
        let old_collections_ids = self.find_document_collections(document_id);

        for collection_id in &old_collections_ids {
            if !collections.contains(collection_id) {
                let document = self.must_get_document(document_id)?;
                let mut old_collection = self.must_get_document(collection_id)?.clone();

                let document_expert = DocumentExpert::new(&self.schema);
                document_expert.remove_document_from_collection(document, &mut old_collection)?;

                self.stage_document(old_collection, &None)?;
            }
        }

        for collection_id in collections {
            if !old_collections_ids.contains(collection_id) {
                let document = self.must_get_document(document_id)?;
                let mut collection = self.must_get_document(collection_id)?.clone();

                let document_expert = DocumentExpert::new(&self.schema);
                document_expert.add_document_to_collection(document, &mut collection)?;

                self.stage_document(collection, &None)?;
            }
        }

        Ok(())
    }

    pub fn erase_document(&mut self, id: &Id) -> Result<()> {
        let mut document = self.must_get_document(id)?.clone();

        document.erase();

        self.stage_document(document, &None)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rs_utils::age::AgeKey;
    use serde_json::json;

    use crate::entities::{new_document, new_empty_document, DocumentLockKey, Id, Revision};

    use super::BazaState;

    #[test]
    fn test_state() {
        let mut state = BazaState::new_test_state();

        assert_eq!(state.get_single_latest_revision(), Revision::INITIAL);
        assert!(!state.is_modified());

        let doc_a1 = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "test": 1 }));
        let mut doc_a3 = doc_a1.clone();
        doc_a3.stage();

        state.insert_snapshots(vec![doc_a1, doc_a2]);
        assert!(state.is_modified());
        assert!(!state.has_staged_documents());
        assert!(state.has_unresolved_conflicts());
        assert_eq!(state.get_latest_revision().len(), 2);

        state.stage_document(doc_a3.clone(), &None).unwrap();
        assert!(state.has_staged_documents());
        assert!(!state.has_unresolved_conflicts());

        state.reset_all_documents().unwrap();
        assert!(!state.has_staged_documents());
        assert!(state.has_unresolved_conflicts());

        state.stage_document(doc_a3.clone(), &None).unwrap();
        state.commit().unwrap();

        let new_rev = Revision::from_value(json!({ "a": 1, "test": 2 })).unwrap();

        assert!(state.get_document(&doc_a3.id).unwrap().is_committed());
        assert_eq!(
            state.get_document(&doc_a3.id).unwrap().get_revision(),
            &new_rev
        );

        assert!(!state.has_staged_documents());
        assert!(!state.has_unresolved_conflicts());
        assert_eq!(state.get_single_latest_revision(), &new_rev);
    }

    #[test]
    fn test_state_read_write() {
        let key = AgeKey::generate_age_x25519_key();
        let mut state = BazaState::new_test_state();

        let id: Id = "test".into();
        state.insert_snapshots(vec![
            new_document(json!({ "test": 1 }))
                .with_rev(json!({ "a": 1 }))
                .with_id(id.clone()),
            new_document(json!({ "test": 2 })).with_rev(json!({ "a": 2, "b": 2 })),
        ]);
        state.lock_document(&id, "test").unwrap();

        let mut data = Cursor::new(Vec::<u8>::new());

        assert!(state.is_modified());
        state.write(&mut data, key.clone()).unwrap();
        assert!(!state.is_modified());
        data.set_position(0);

        let state1 = BazaState::read(&mut data, key.clone(), state.schema).unwrap();

        assert_eq!(state.file, state1.file);
    }

    #[test]
    fn test_state_stage_locks() {
        let mut state = BazaState::new_test_state();

        let doc1 = new_empty_document().with_rev(json!({ "a": 1 }));

        state.insert_snapshot(doc1.clone()).unwrap();

        assert!(state
            .stage_document(
                doc1.clone(),
                &Some(DocumentLockKey::from_string("unexpected key"))
            )
            .is_err());

        state.stage_document(doc1.clone(), &None).unwrap();

        let key = state
            .lock_document(&doc1.id, "test")
            .unwrap()
            .get_key()
            .clone();

        assert!(state.stage_document(doc1.clone(), &None).is_err());
        assert!(state
            .stage_document(
                doc1.clone(),
                &Some(DocumentLockKey::from_string("wrong key"))
            )
            .is_err());

        state.stage_document(doc1.clone(), &Some(key)).unwrap();

        assert!(state.commit().is_err());
    }

    #[test]
    fn test_reset_document() {
        let mut state = BazaState::new_test_state();

        let doc1 = new_empty_document();

        state.stage_document(doc1.clone(), &None).unwrap();
        state.modified = false;

        state.reset_document(&doc1.id, &None).unwrap();
        assert!(state.is_modified());

        let doc2 = new_empty_document().with_rev(json!({ "a": 1 }));
        state.insert_snapshot(doc2.clone()).unwrap();
        state.stage_document(doc2.clone(), &None).unwrap();

        state.modified = false;
        state.reset_document(&doc2.id, &None).unwrap();
        assert!(state.is_modified());
    }
}
