mod baza_paths;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io::Read,
};

use anyhow::{anyhow, bail, ensure, Context, Result};

use rs_utils::{crypto_key::CryptoKey, file_exists, log, FsTransaction};

use crate::{
    entities::{BLOBId, Document, Id, InstanceId, Revision},
    schema::DataSchema,
};

use baza_paths::BazaPaths;

use super::{
    baza_storage::{
        create_empty_storage_file, merge_storages_to_file, BazaDocumentKey, BazaFileStorage,
        STORAGE_VERSION,
    },
    BazaInfo, BazaState, BazaStorage,
};

// create?
// on startup:
// * read baza state
// * merge all baza storage files into 1
// * read (if no local changes)? baza storage info
// * what if baza storage is newer than baza state? - pull changes
// on commit:
// * acquire write lock on lockfile
// * increment baza_rev
// * update revision on local documents
// * push updated documents to baza storage
// * commit changes

#[derive(Clone)]
pub struct BazaManagerOptions {
    pub storage_dir: String,
    pub state_dir: String,
    pub key: CryptoKey,
    pub schema: DataSchema,
}

impl BazaManagerOptions {
    #[cfg(test)]
    pub fn new_for_tests(test_dir: &str) -> Self {
        let key = CryptoKey::new_random_key();
        let schema = DataSchema::new_test_schema();

        BazaManagerOptions {
            storage_dir: format!("{test_dir}/storage"),
            state_dir: format!("{test_dir}/state"),
            key,
            schema,
        }
    }
}

pub struct BazaManager {
    state: RefCell<BazaState>,
    paths: BazaPaths,
    key: CryptoKey,
    info: BazaInfo,
}

impl BazaManager {
    pub fn new(options: BazaManagerOptions) -> Result<Self> {
        let paths = BazaPaths::new(options.storage_dir, options.state_dir);
        let schema = options.schema;
        let key = options.key;

        log::info!("Opening baza {paths}");

        paths.ensure_dirs_exist()?;

        let info = BazaInfo {
            name: schema.get_app_name().to_string(),
            data_version: schema.get_latest_data_version(),
            storage_version: STORAGE_VERSION,
        };

        let state = if file_exists(&paths.state_file)? {
            let state = BazaState::read_file(&paths.state_file, &key)?;

            ensure!(state.get_info() == &info, "State info mismatch");

            log::info!("Read state file in {}", paths.state_file);

            state
        } else {
            // create state if necessary
            let state = BazaState::new(InstanceId::generate(), info.clone(), HashMap::new());
            state.write_to_file(&paths.state_file, &key)?;

            log::info!("Created new state file in {}", paths.state_file);

            state
        };

        // create main storage file if necessary
        if !file_exists(&paths.storage_main_db_file)? {
            create_empty_storage_file(&paths.storage_main_db_file, &key, &info)?;

            log::info!(
                "Created new main storage file {}",
                paths.storage_main_db_file
            );
        }

        let baza_manager = Self {
            state: RefCell::new(state),
            key,
            info,
            paths,
        };

        baza_manager.merge_storages()?;
        baza_manager.update_state_from_storage()?;

        Ok(baza_manager)
    }

    #[cfg(test)]
    pub fn create_storage_file(&self, file_path: &str, docs: &[Document]) {
        use rs_utils::create_file_writer;

        use crate::baza2::baza_storage::create_storage;

        let mut storage_writer = create_file_writer(file_path, false).unwrap();
        create_storage(&mut storage_writer, &self.key, &self.info, docs).unwrap();
    }

    fn get_local_blob_path(&self, id: &BLOBId) -> String {
        self.paths.get_state_blob_path(id)
    }

    pub fn get_blob_path(&self, id: &BLOBId) -> Result<String> {
        let blob_path = self.get_local_blob_path(id);

        if file_exists(&blob_path)? {
            return Ok(blob_path);
        }

        let blob_path = self.paths.get_storage_blob_path(id);
        if file_exists(&blob_path)? {
            return Ok(blob_path);
        }

        bail!("Coud't find blob {id}")
    }

    pub fn list_blobs(&self) -> Result<HashSet<BLOBId>> {
        self.paths.list_blobs()
    }

    fn open_storage(&self, file_path: &str) -> Result<BazaFileStorage<'_>> {
        BazaStorage::read_file(file_path, &self.key)
    }

    pub fn commit(&self) -> Result<()> {
        // FIXME use read/write locks

        let mut state = self.state.borrow_mut();

        if !state.has_staged_documents() {
            return Ok(());
        }

        self.merge_storages()?;

        let new_blobs = self
            .paths
            .list_state_blobs()?
            .into_iter()
            .map(|blob_id| {
                let blob_path = self.get_local_blob_path(&blob_id);

                (blob_id, blob_path)
            })
            .collect::<HashMap<_, _>>();

        let mut tx = FsTransaction::new();

        // backup db file
        let old_db_file = tx.move_to_backup(self.paths.storage_main_db_file.clone())?;

        // open old db file
        let storage = self.open_storage(&old_db_file)?;

        // update state
        state.commit()?;

        // collect snapshots that aren't present in the storage
        let new_documents = state
            .iter_documents()
            .flat_map(|head| head.iter_snapshots())
            .filter(|document| !storage.contains(&BazaDocumentKey::for_document(document)))
            .collect::<Vec<_>>();

        // write changes to db file
        storage
            .add_and_save_to_file(&self.paths.storage_main_db_file, new_documents.into_iter())?;

        // move blobs
        for (new_blob_id, file_path) in new_blobs {
            tx.move_file(
                file_path,
                self.paths.get_storage_blob_path(&new_blob_id),
                true,
            )?;
        }

        // backup state file
        tx.move_to_backup(self.paths.state_file.clone())?;

        // write changes to state file
        state.write_to_file(&self.paths.state_file, &self.key)?;

        tx.commit()?;

        drop(state);

        self.update_state_from_storage()?;

        Ok(())
    }

    fn update_state_from_storage(&self) -> Result<()> {
        let mut state = self.state.borrow_mut();

        let mut storage = self.open_storage(&self.paths.storage_main_db_file)?;

        let latest_snapshots_count = update_state_from_storage(&mut state, &mut storage)?;

        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");
            state.write_to_file(&self.paths.state_file, &self.key)?;
        }

        Ok(())
    }

    fn merge_storages(&self) -> Result<()> {
        let db_files = self.paths.list_storage_db_files()?;

        if db_files.is_empty() {
            log::debug!("No existing db files found");
            return Ok(());
        }

        let main_db_file = &self.paths.storage_main_db_file;
        if db_files.len() == 1 && db_files[0] == *main_db_file {
            log::debug!("There's only main db file");
            return Ok(());
        }

        // if more than 1 storage
        // or if no main storage file

        log::info!("Merging {} db files into one", db_files.len());

        let mut tx = FsTransaction::new();

        // backup db files and open storages
        let storages = db_files
            .iter()
            .map(|db_file| {
                let new_db_file = tx.move_to_backup(db_file)?;

                self.open_storage(&new_db_file)
                    .context(anyhow!("Failed to open storage for db {db_file}"))
            })
            .collect::<Result<Vec<_>>>()?;

        merge_storages_to_file(&self.info, storages, main_db_file)?;

        tx.commit()?;

        Ok(())
    }
}

fn add_keys<'r>(
    keys: &mut HashSet<BazaDocumentKey>,
    id: &Id,
    revs: impl Iterator<Item = &'r &'r Revision>,
) {
    keys.extend(revs.map(|rev| BazaDocumentKey::new(id.clone(), (*rev).clone())));
}

fn update_state_from_storage<R: Read>(
    state: &mut BazaState,
    storage: &mut BazaStorage<R>,
) -> Result<usize> {
    if state.has_staged_documents() {
        return Ok(0);
    }

    let storage_info = storage.get_info()?;
    ensure!(
        storage_info == state.get_info(),
        "state info and storage info must match"
    );

    let mut latest_snapshot_keys: HashSet<BazaDocumentKey> = HashSet::new();

    // compare storage index with state
    for (id, index_revs) in storage.index.as_index_map() {
        let index_rev = index_revs
            .iter()
            .next()
            .context("index revs must not be empty")?;

        let document_head = if let Some(document_head) = state.get_document(id) {
            document_head
        } else {
            add_keys(&mut latest_snapshot_keys, id, index_revs.iter());
            continue;
        };

        ensure!(
            document_head.is_committed(),
            "Document {id} must be committed"
        );

        let state_revs = document_head.get_revision();
        let state_rev = state_revs
            .iter()
            .next()
            .context("state revs must not be empty")?;

        if state_rev > index_rev {
            continue;
        }

        if state_rev < index_rev {
            add_keys(&mut latest_snapshot_keys, id, index_revs.iter());
            continue;
        }

        // conflicting revs
        add_keys(
            &mut latest_snapshot_keys,
            id,
            index_revs.difference(&state_revs),
        );
    }

    let latest_snapshots_count = latest_snapshot_keys.len();

    // read documents from storage & update state if needed
    while !latest_snapshot_keys.is_empty() {
        let (ref key, ref raw_document) = storage.next().context("No records in the storage")??;

        if !latest_snapshot_keys.contains(key) {
            continue;
        }

        let document: Document =
            serde_json::from_str(raw_document).context("Failed to parse raw document")?;

        state.insert_snapshot(document)?;

        latest_snapshot_keys.remove(key);
    }

    Ok(latest_snapshots_count)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use rs_utils::{crypto_key::CryptoKey, dir_exists, file_exists, TempFile};

    use crate::{
        baza2::{
            baza_manager::BazaManagerOptions, baza_storage::create_test_storage, DocumentHead,
        },
        tests::new_document,
    };

    use super::{update_state_from_storage, BazaManager, BazaState};

    #[test]
    fn test_update_state_from_storage() {
        let key = CryptoKey::new_random_key();

        let doc_a = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a1 = doc_a.clone().with_rev(json!({ "b": 1 }));

        let doc_b = new_document(json!({})).with_rev(json!({ "b": 1 }));
        let doc_b1 = doc_b.clone().with_rev(json!({ "b": 2 }));

        let doc_c = new_document(json!({})).with_rev(json!({ "c": 3 }));

        let mut state = BazaState::new_test_state();
        state.insert_snapshots(vec![doc_a.clone(), doc_b.clone()]);

        let mut storage = create_test_storage(
            &key,
            &vec![
                doc_a.clone(),
                doc_a1.clone(),
                doc_b.clone(),
                doc_b1.clone(),
                doc_c.clone(),
            ],
        );

        let changes = update_state_from_storage(&mut state, &mut storage).unwrap();
        assert_eq!(changes, 3);

        assert_eq!(
            *state.get_document(&doc_a.id).unwrap(),
            DocumentHead::new_conflict([doc_a.clone(), doc_a1.clone(),].into_iter()).unwrap(),
        );

        assert_eq!(
            *state.get_document(&doc_b.id).unwrap(),
            DocumentHead::new(doc_b1),
        );

        assert_eq!(
            *state.get_document(&doc_c.id).unwrap(),
            DocumentHead::new(doc_c),
        );
    }

    #[test]
    fn test_baza_manager() {
        let temp_dir = TempFile::new_with_details("test_baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new(BazaManagerOptions::new_for_tests(&temp_dir.path)).unwrap();

        assert!(dir_exists(&manager.paths.storage_dir).unwrap());
        assert!(dir_exists(&manager.paths.state_dir).unwrap());

        assert!(file_exists(&manager.paths.state_file).unwrap());
        assert!(file_exists(&manager.paths.storage_main_db_file).unwrap());

        {
            let mut state = manager.state.borrow_mut();

            state.stage_document(new_document(json!({}))).unwrap();

            assert!(state.has_staged_documents());

            state
                .insert_snapshot(new_document(json!({})).with_rev(json!({ "a": 1 })))
                .unwrap();
        }

        manager.commit().unwrap();

        {
            let state = manager.state.borrow();
            assert!(!state.has_staged_documents());

            let storage = manager
                .open_storage(&manager.paths.storage_main_db_file)
                .unwrap();
            assert_eq!(storage.index.len(), 2);
        }

        // TODO check if commits, including BLOBs
    }

    #[test]
    fn test_baza_manager_merges_storages() {
        let temp_dir = TempFile::new_with_details("test_baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new(BazaManagerOptions::new_for_tests(&temp_dir.path)).unwrap();

        let db_file_1 = manager.paths.get_storage_file("db1");
        let db_file_2 = manager.paths.get_storage_file("db2");

        let doc_a1 = new_document(json!({ "test": "a" })).with_rev(json!({ "a": 1 }));
        let doc_a2 = doc_a1.clone().with_rev(json!({ "a": 2 }));

        let doc_b1 = new_document(json!({ "test": "b" })).with_rev(json!({ "a": 1 }));

        manager.create_storage_file(&db_file_1, &[doc_a1.clone(), doc_b1.clone()]);
        manager.create_storage_file(&db_file_2, &[doc_a1.clone(), doc_a2.clone()]);

        assert!(file_exists(&db_file_1).unwrap());
        assert!(file_exists(&db_file_2).unwrap());

        {
            let mut state = manager.state.borrow_mut();

            state.stage_document(new_document(json!({}))).unwrap();
        }

        manager.commit().unwrap();

        assert!(!file_exists(&db_file_1).unwrap());
        assert!(!file_exists(&db_file_2).unwrap());

        {
            let state = manager.state.borrow();
            assert_eq!(state.iter_documents().count(), 3);

            let storage = manager
                .open_storage(&manager.paths.storage_main_db_file)
                .unwrap();
            assert_eq!(storage.index.len(), 4);
        }
    }
}
