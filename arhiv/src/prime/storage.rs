use crate::common::{PathFinder, StateDTO, StorageState};
use crate::entities::*;
use anyhow::*;
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub struct Storage {
    pf: PathFinder,
    state: StorageState,
}

impl Storage {
    pub fn open(root_path: &str) -> Result<Storage> {
        let pf = PathFinder::new(root_path.to_string());
        pf.assert_dirs_exist()?;

        let state = StorageState::new(root_path);
        state.asset_exists()?;

        // TODO lock file

        Ok(Storage { pf, state })
    }

    pub fn create(root_path: &str) -> Result<Storage> {
        let path = Path::new(root_path);

        if !path.is_absolute() {
            return Err(anyhow!("path must be absolute: {}", root_path));
        }

        if path.exists() {
            return Err(anyhow!("path already exists: {}", root_path));
        }

        let pf = PathFinder::new(root_path.to_string());
        pf.create_dirs()?; // create required dirs

        let state = StorageState::new(root_path);
        state.init()?; // create state file

        let prime = Storage { pf, state };

        println!("created arhiv prime in {}", root_path);

        Ok(prime)
    }
}
