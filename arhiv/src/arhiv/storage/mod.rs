use crate::entities::*;
use anyhow::*;
use path_manager::PathManager;
pub use queries::*;
pub use query_params::*;
use rusqlite::{Connection, OpenFlags};

mod path_manager;
mod queries;
mod query_params;
mod utils;

pub struct Storage {
    path_manager: PathManager,
}

impl Storage {
    pub fn open(root_path: &str) -> Result<Storage> {
        let path_manager = PathManager::new(root_path.to_string());
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(Storage { path_manager })
    }

    pub fn create(root_path: &str) -> Result<Storage> {
        let path_manager = PathManager::new(root_path.to_string());
        path_manager.create_dirs()?;

        let conn = Connection::open(path_manager.get_db_file())?;
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(Storage { path_manager })
    }

    pub fn get_connection(&self) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            self.path_manager.get_db_file(),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;

        Ok(conn)
    }

    pub fn get_writable_connection(&self) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            self.path_manager.get_db_file(),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;

        Ok(conn)
    }

    pub fn get_attachment_file_path(&self, id: &Id) -> String {
        format!("{}/{}", self.path_manager.get_data_directory(), id)
    }
}
