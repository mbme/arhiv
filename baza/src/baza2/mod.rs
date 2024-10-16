use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
};

use anyhow::{bail, ensure, Result};

use rs_utils::{
    create_file_reader, create_file_writer, crypto_key::CryptoKey, file_exists, FsTransaction,
};

pub use baza_state::BazaState;
pub use baza_storage::{BazaInfo, BazaStorage};

use crate::{
    entities::{BLOBId, InstanceId},
    get_local_blob_ids,
    path_manager::PathManager,
};

mod baza_state;
mod baza_storage;

// create?
// on startup:
// * read baza state
// * read (if no local changes)? baza storage info
// * what if baza storage is newer than baza state? - pull changes
// on commit:
// * acquire write lock on lockfile
// * increment baza_rev
// * update revision on local documents
// * push updated documents to baza storage
// * commit changes

pub struct BazaManager {
    instance_id: InstanceId,
    state: RefCell<BazaState>,
    path_manager: PathManager,
    key: CryptoKey,
}

impl BazaManager {
    pub fn borrow(&self) -> Ref<'_, BazaState> {
        self.state.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, BazaState> {
        self.state.borrow_mut()
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

    pub fn commit(self) -> Result<Self> {
        // FIXME use read/write locks

        let mut state = self.borrow_mut();

        ensure!(
            !state.has_unresolved_conflicts(),
            "Can't commit with unresolved conflicts"
        );

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
        let reader = create_file_reader(&old_db_file)?;
        let storage = BazaStorage::read(reader, self.key.clone())?;

        // collect changed documents & update state
        let new_documents = state.commit(&self.instance_id)?;

        // write changes to db file
        let writer = create_file_writer(&self.path_manager.db2_file)?;
        storage.add(writer, new_documents)?;

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
        let writer = create_file_writer(&self.path_manager.state_file)?;
        state.write(writer)?;

        tx.commit()?;

        drop(state);

        Ok(self)
    }

    // fn update(&mut self, update: BazaUpdate) -> Result<()> {
    //     todo!()
    // }

    // TODO pull changes from Storage into State
}
