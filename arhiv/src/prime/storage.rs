use super::state::{StateDTO, StorageState};
use crate::common::PathFinder;
use anyhow::*;

pub struct Storage {
    pub pf: PathFinder,
    pub state: StorageState,
}

impl Storage {
    pub fn open(root_path: &str) -> Result<Storage> {
        let pf = PathFinder::new(root_path.to_string());
        pf.assert_dirs_exist()?;

        let state = StorageState::new(root_path);
        state.assert_exists()?;

        // TODO lock file

        Ok(Storage { pf, state })
    }

    pub fn create(root_path: &str) -> Result<Storage> {
        let pf = PathFinder::new(root_path.to_string());
        pf.create_dirs()?; // create required dirs

        let state = StorageState::new(root_path);
        state.write(StateDTO { rev: 0 })?;

        let replica = Storage { pf, state };

        println!("created arhiv storage in {}", root_path);

        Ok(replica)
    }
}
