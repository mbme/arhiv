use std::{collections::HashSet, time::Instant};

use anyhow::{Context, Result, ensure};

use rs_utils::{Timestamp, age::AgeKey, log};

pub use self::search::SearchEngine;
use self::state_file::BazaStateFile;
use crate::{
    BazaInfo, DocumentExpert,
    entities::{
        Document, DocumentLock, DocumentLockKey, Id, InstanceId, LatestRevComputer, Revision,
    },
    schema::DataSchema,
};

mod document_head;
mod document_locks_file;
mod query;
mod refs;
mod search;
mod state_file;

pub use document_head::DocumentHead;
use document_locks_file::DocumentLocksFile;
pub use document_locks_file::Locks;
pub use query::{Filter, ListPage};

use super::baza_paths::BazaPaths;

pub struct BazaState {
    file: BazaStateFile,
    schema: DataSchema,
    search: SearchEngine,
    document_locks: DocumentLocksFile,
}

impl BazaState {
    pub fn new(instance_id: InstanceId, info: BazaInfo, schema: DataSchema) -> Self {
        BazaState {
            file: BazaStateFile::new(instance_id, info),
            search: SearchEngine::new(schema.clone()),
            schema,
            document_locks: DocumentLocksFile::new(),
        }
    }

    #[cfg(test)]
    pub fn new_test_state() -> Self {
        Self::new(
            InstanceId::from_string("test").unwrap(),
            BazaInfo::new_test_info(),
            DataSchema::new_test_schema(),
        )
    }

    pub fn read(paths: &BazaPaths, key: AgeKey, schema: DataSchema) -> Result<Self> {
        let file = BazaStateFile::read(&paths.state_file, key.clone())?;

        let search =
            match SearchEngine::read(&paths.state_search_index_file, key.clone(), schema.clone()) {
                Ok(search) => search,
                Err(err) => {
                    log::debug!("Failed to read search index: {err}");

                    let mut search = SearchEngine::new(schema.clone());

                    let start_time = Instant::now();
                    for head in file.documents.values() {
                        search.index_document(head.get_single_document())?;
                    }
                    let duration = start_time.elapsed();
                    log::info!(
                        "Built search index of {} documents in {:?}",
                        file.documents.len(),
                        duration
                    );

                    search.write(&paths.state_search_index_file, key.clone())?;

                    search
                }
            };

        let locks = match DocumentLocksFile::read(&paths.state_document_locks_file, key.clone()) {
            Ok(locks) => locks,
            Err(err) => {
                log::debug!("Failed to read document locks file: {err}");

                let mut locks = DocumentLocksFile::new();

                locks.write(&paths.state_document_locks_file, key.clone())?;

                locks
            }
        };

        Ok(BazaState {
            file,
            search,
            schema,
            document_locks: locks,
        })
    }

    pub fn write(&mut self, paths: &BazaPaths, key: AgeKey) -> Result<()> {
        if self.file.modified {
            self.file.write(&paths.state_file, key.clone())?;
            self.file.modified = false;
        }

        if self.search.is_modified() {
            self.search
                .write(&paths.state_search_index_file, key.clone())?;
        }

        if self.document_locks.is_modified() {
            self.document_locks
                .write(&paths.state_document_locks_file, key)?;
        }

        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.file.modified || self.search.is_modified() || self.document_locks.is_modified()
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
            let document_revs = document.iter_original_revs();
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
            .flat_map(|head| head.iter_original_revs());

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
        document: Document,
        lock_key: &Option<DocumentLockKey>,
    ) -> Result<&Document> {
        let id = document.id.clone();

        self.document_locks.check_document_lock(&id, lock_key)?;

        let current_value = self.file.documents.remove(&id);

        let updated_head = if let Some(mut document_head) = current_value {
            document_head.modify(document)?;
            document_head
        } else {
            DocumentHead::new_staged(document)
        };

        self.update_document_refs(&updated_head)?;
        self.search
            .index_document(updated_head.get_single_document())?;
        self.file.documents.insert(id.clone(), updated_head);
        self.file.modified = true;
        log::trace!("State modified: staged document");

        let document = self
            .get_document(&id)
            .context("Document must exist")?
            .get_single_document();

        Ok(document)
    }

    pub fn insert_document_head(&mut self, head: DocumentHead) -> Result<()> {
        let current_value = self.file.documents.remove(head.get_id());

        if let Some(document_head) = current_value {
            ensure!(
                !document_head.is_staged(),
                "Can't insert into staged document"
            );
        }

        self.update_document_refs(&head)?;
        self.search.index_document(head.get_single_document())?;

        self.file.documents.insert(head.get_id().clone(), head);
        self.file.modified = true;

        log::trace!("State modified: inserted document head");

        Ok(())
    }

    #[cfg(test)]
    pub fn insert_snapshots(&mut self, documents: Vec<Document>) {
        use std::collections::HashMap;

        let mut grouped_documents: HashMap<Id, Vec<Document>> = HashMap::new();

        for document in documents {
            grouped_documents
                .entry(document.id.clone())
                .or_insert_with(Vec::new)
                .push(document);
        }

        for docs in grouped_documents.into_values() {
            let document_head = DocumentHead::new(docs.into_iter()).unwrap();

            self.insert_document_head(document_head)
                .expect("must insert document head");
        }
    }

    pub(super) fn update_snapshots_count(&mut self, id: &Id, snapshots_count: usize) -> Result<()> {
        let head = self.get_mut_document(id).expect("must find document");

        if head.get_snapshots_count() != snapshots_count {
            head.update_snapshots_count(snapshots_count);
            self.file.modified = true;
            log::trace!("State modified: updated snapshots count");
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

    pub fn has_conflicts(&self) -> bool {
        self.iter_documents()
            .any(|document_head| document_head.is_conflict())
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
        self.document_locks.check_document_lock(id, lock_key)?;

        let (id, document) = self
            .file
            .documents
            .remove_entry(id)
            .context("Document doesn't exist")?;
        self.remove_document_refs(&id);
        self.search.remove_document_index(&id);

        if let Some(updated_head) = document.reset() {
            self.update_document_refs(&updated_head)?;
            self.search
                .index_document(updated_head.get_single_document())?;
            self.file.documents.insert(id, updated_head);
        }

        self.file.modified = true;
        log::trace!("State modified: reset document");

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
            self.search
                .index_document(updated_head.get_single_document())?;
            self.file.documents.insert(id, updated_head);
        }

        self.file.modified = true;
        log::trace!("State modified: commit");

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

    pub fn list_document_locks(&self) -> &Locks {
        self.document_locks.list_document_locks()
    }

    pub fn has_document_locks(&self) -> bool {
        !self.list_document_locks().is_empty()
    }

    pub fn is_document_locked(&self, id: &Id) -> bool {
        self.document_locks.is_document_locked(id)
    }

    pub fn lock_document(&mut self, id: &Id, reason: impl Into<String>) -> Result<&DocumentLock> {
        ensure!(
            !self.document_locks.is_document_locked(id),
            "document {id} already locked"
        );

        let document = self.get_document(id);
        ensure!(document.is_some(), "document {id} doesn't exist");

        let lock = self.document_locks.lock_document(id, reason.into())?;

        Ok(lock)
    }

    pub fn unlock_document(&mut self, id: &Id, key: &DocumentLockKey) -> Result<()> {
        self.document_locks.unlock_document(id, key)
    }

    pub fn unlock_document_without_key(&mut self, id: &Id) -> Result<()> {
        self.document_locks.unlock_document_without_key(id)
    }
}

#[cfg(test)]
mod tests {
    use rs_utils::{TempFile, age::AgeKey};
    use serde_json::json;

    use crate::{
        BazaPaths,
        entities::{DocumentLockKey, Id, Revision, new_document, new_empty_document},
    };

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
            state
                .get_document(&doc_a3.id)
                .unwrap()
                .get_single_revision()
                .unwrap(),
            &new_rev
        );

        assert!(!state.has_staged_documents());
        assert!(!state.has_unresolved_conflicts());
        assert_eq!(state.get_single_latest_revision(), &new_rev);
    }

    #[test]
    fn test_state_read_write() {
        let temp_dir = TempFile::new_with_details("test_baza", "");
        temp_dir.mkdir().unwrap();

        let paths = BazaPaths::new_for_tests(&temp_dir.path);
        paths.ensure_dirs_exist().unwrap();

        let key = AgeKey::generate_age_x25519_key();
        let mut state = BazaState::new_test_state();

        let id: Id = "test".into();
        state.insert_snapshots(vec![
            new_document(json!({ "test": "1" }))
                .with_rev(json!({ "a": 1 }))
                .with_id(id.clone()),
            new_document(json!({ "test": "2" })).with_rev(json!({ "a": 2, "b": 2 })),
        ]);
        state.lock_document(&id, "test").unwrap();

        assert!(state.is_modified());
        state.write(&paths, key.clone()).unwrap();
        assert!(!state.is_modified());

        let state1 = BazaState::read(&paths, key.clone(), state.schema).unwrap();

        assert_eq!(state.file, state1.file);
    }

    #[test]
    fn test_state_stage_locks() {
        let mut state = BazaState::new_test_state();

        let doc1 = new_empty_document().with_rev(json!({ "a": 1 }));

        state.insert_snapshots(vec![doc1.clone()]);

        assert!(
            state
                .stage_document(
                    doc1.clone(),
                    &Some(DocumentLockKey::from_string("unexpected key"))
                )
                .is_err()
        );

        state.stage_document(doc1.clone(), &None).unwrap();

        let key = state
            .lock_document(&doc1.id, "test")
            .unwrap()
            .get_key()
            .clone();

        assert!(state.stage_document(doc1.clone(), &None).is_err());
        assert!(
            state
                .stage_document(
                    doc1.clone(),
                    &Some(DocumentLockKey::from_string("wrong key"))
                )
                .is_err()
        );

        state.stage_document(doc1.clone(), &Some(key)).unwrap();

        assert!(state.commit().is_err());
    }

    #[test]
    fn test_reset_document() {
        let mut state = BazaState::new_test_state();

        let doc1 = new_empty_document();

        state.stage_document(doc1.clone(), &None).unwrap();
        state.file.modified = false;

        state.reset_document(&doc1.id, &None).unwrap();
        assert!(state.is_modified());

        let doc2 = new_empty_document().with_rev(json!({ "a": 1 }));
        state.insert_snapshots(vec![doc2.clone()]);
        state.stage_document(doc2.clone(), &None).unwrap();

        state.file.modified = false;
        state.reset_document(&doc2.id, &None).unwrap();
        assert!(state.is_modified());
    }

    #[test]
    fn test_locks() {
        let mut state = BazaState::new_test_state();

        assert!(state.list_document_locks().is_empty());

        let doc1 = new_empty_document();
        assert!(!state.is_document_locked(&doc1.id));
        assert!(state.lock_document(&doc1.id, "test").is_err());
        assert!(
            state
                .unlock_document(&doc1.id, &DocumentLockKey::from_string("some key"))
                .is_err()
        );
        assert!(state.unlock_document_without_key(&doc1.id).is_err());

        state.stage_document(doc1.clone(), &None).unwrap();

        state.file.modified = false;
        let key = state
            .lock_document(&doc1.id, "test")
            .unwrap()
            .get_key()
            .clone();
        assert!(state.is_modified());
        assert!(state.lock_document(&doc1.id, "test").is_err());

        assert!(state.has_document_locks());
        assert!(state.is_document_locked(&doc1.id));

        assert!(
            state
                .unlock_document(&doc1.id, &DocumentLockKey::from_string("wrong"))
                .is_err()
        );

        state.file.modified = false;
        state.unlock_document(&doc1.id, &key).unwrap();
        assert!(state.is_modified());
        assert!(state.unlock_document(&doc1.id, &key).is_err());
        assert!(state.unlock_document_without_key(&doc1.id).is_err());

        assert!(!state.has_document_locks());
        assert!(!state.is_document_locked(&doc1.id));

        state.lock_document(&doc1.id, "test").unwrap();
        state.unlock_document_without_key(&doc1.id).unwrap();

        assert!(!state.has_document_locks());
        assert!(!state.is_document_locked(&doc1.id));
    }
}
