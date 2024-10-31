use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use anyhow::{anyhow, bail, ensure, Context, Result};

use baza_storage::BazaDocumentKey;
use rs_utils::{
    create_file_reader, create_file_writer, crypto_key::CryptoKey, file_exists, log, FsTransaction,
};

pub use baza_state::BazaState;
pub use baza_storage::{BazaInfo, BazaStorage};

use crate::{
    entities::{BLOBId, Document, Id, Revision},
    get_local_blob_ids,
    path_manager::PathManager,
};

mod baza_state;
mod baza_storage;

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

pub struct BazaManager {
    state: RefCell<BazaState>,
    path_manager: PathManager,
    key: CryptoKey,
}

impl BazaManager {
    pub fn new(path_manager: PathManager, key: CryptoKey) -> Result<Self> {
        let state_reader = create_file_reader(&path_manager.state_file)?;
        let state = BazaState::read(state_reader)?;

        let mut baza_manager = Self {
            state: RefCell::new(state),
            path_manager,
            key,
        };

        baza_manager.merge_storages()?;
        baza_manager.sync_state_with_storage()?;

        Ok(baza_manager)
    }

    fn get_local_blob_path(&self, id: &BLOBId) -> String {
        self.path_manager.get_state_blob_path(id)
    }

    pub fn get_blob_path(&self, id: &BLOBId) -> Result<String> {
        let blob_path = self.get_local_blob_path(id);

        if file_exists(&blob_path)? {
            return Ok(blob_path);
        }

        let blob_path = self.path_manager.get_db2_blob_path(id);
        if file_exists(&blob_path)? {
            return Ok(blob_path);
        }

        bail!("Coud't find blob {id}")
    }

    fn list_local_blobs(&self) -> Result<HashSet<BLOBId>> {
        get_local_blob_ids(&self.path_manager.state_data_dir)
    }

    pub fn list_blobs(&self) -> Result<HashSet<BLOBId>> {
        let mut ids = get_local_blob_ids(&self.path_manager.db2_data_dir)?;
        let local_ids = self.list_local_blobs()?;

        ids.extend(local_ids);

        Ok(ids)
    }

    pub fn commit(mut self) -> Result<Self> {
        // FIXME use read/write locks

        self.merge_storages()?;

        let mut state = self.state.borrow_mut();

        if !state.is_modified() {
            drop(state);

            return Ok(self);
        }

        let new_blobs = self
            .list_local_blobs()?
            .into_iter()
            .map(|blob_id| {
                let blob_path = self.get_local_blob_path(&blob_id);

                (blob_id, blob_path)
            })
            .collect::<HashMap<_, _>>();

        let mut tx = FsTransaction::new();

        // backup db file
        let old_db_file = tx.move_to_backup(self.path_manager.db2_file.clone())?;

        // open old db file
        let storage_reader = create_file_reader(&old_db_file)?;
        let storage = BazaStorage::read(storage_reader, self.key.clone())?;

        // collect changed documents & update state
        let new_documents = state.commit()?;

        // write changes to db file
        let storage_writer = create_file_writer(&self.path_manager.db2_file)?;
        storage.add(storage_writer, new_documents)?;

        // move blobs
        for (new_blob_id, file_path) in new_blobs {
            tx.move_file(
                file_path,
                self.path_manager.get_db2_blob_path(&new_blob_id),
                true,
            )?;
        }

        // backup state file
        tx.move_to_backup(self.path_manager.state_file.clone())?;

        // write changes to state file
        let state_writer = create_file_writer(&self.path_manager.state_file)?;
        state.write(state_writer)?;

        tx.commit()?;

        drop(state);

        self.sync_state_with_storage()?;

        Ok(self)
    }

    fn sync_state_with_storage(&mut self) -> Result<()> {
        let mut state = self.state.borrow_mut();

        if state.is_modified() {
            return Ok(());
        }

        let storage_reader = create_file_reader(&self.path_manager.db2_file)?;
        let mut storage = BazaStorage::read(storage_reader, self.key.clone())?;

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
        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");
        }

        // read documents from storage & update state if needed
        while !latest_snapshot_keys.is_empty() {
            let (ref key, ref raw_document) =
                storage.next().context("No records in the storage")??;

            if !latest_snapshot_keys.contains(key) {
                continue;
            }

            let document: Document =
                serde_json::from_str(raw_document).context("Failed to parse raw document")?;

            state.insert_document(document)?;

            latest_snapshot_keys.remove(key);
        }

        if latest_snapshots_count > 0 {
            let state_writer = create_file_writer(&self.path_manager.state_file)?;
            state.write(state_writer)?;
        }

        Ok(())
    }

    fn merge_storages(&self) -> Result<()> {
        let db_files = self.path_manager.list_db2_baza_files()?;

        if db_files.is_empty() {
            log::debug!("No existing db files found");
            return Ok(());
        }

        let main_db_file = &self.path_manager.db2_file;
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
                let storage_reader = create_file_reader(&new_db_file)?;

                BazaStorage::read(storage_reader, self.key.clone())
                    .context(anyhow!("Failed to open storage for db {db_file}"))
            })
            .collect::<Result<Vec<_>>>()?;

        let storage_writer = create_file_writer(main_db_file)?;
        BazaStorage::merge_all(storages, storage_writer)?;

        tx.commit()?;

        Ok(())
    }

    // fn update(&mut self, update: BazaUpdate) -> Result<()> {
    //     todo!()
    // }
}

fn add_keys<'r>(
    keys: &mut HashSet<BazaDocumentKey>,
    id: &Id,
    revs: impl Iterator<Item = &'r &'r Revision>,
) {
    keys.extend(revs.map(|rev| BazaDocumentKey::new(id.clone(), (*rev).clone())));
}
