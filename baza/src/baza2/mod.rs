use std::cell::{Ref, RefCell, RefMut};

use anyhow::{bail, Result};

use rs_utils::file_exists;

pub use baza_state::BazaState;
pub use baza_storage::{BazaInfo, BazaStorage};

use crate::{entities::BLOBId, path_manager::PathManager};

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
    state: RefCell<BazaState>,
    path_manager: PathManager,
}

impl BazaManager {
    pub fn borrow(&self) -> Ref<'_, BazaState> {
        self.state.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, BazaState> {
        self.state.borrow_mut()
    }

    pub fn commit(&mut self) -> Result<()> {
        todo!()
    }

    pub fn get_blob_path(&self, id: &BLOBId) -> Result<String> {
        let blob_path = self.path_manager.get_state_blob_path(id);

        if file_exists(&blob_path)? {
            return Ok(blob_path);
        }

        let blob_path = self.path_manager.get_db2_blob_path(id);
        if file_exists(&blob_path)? {
            return Ok(blob_path);
        }

        bail!("Coud't find blob {id}")
    }

    // fn update(&mut self, update: BazaUpdate) -> Result<()> {
    //     todo!()
    // }

    // TODO pull changes from Storage into State
}
