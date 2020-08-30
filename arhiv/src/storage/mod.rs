use crate::config::Config;
use crate::entities::*;
use anyhow::*;
pub use attachment_data::AttachmentData;
pub use connection::*;
use path_manager::PathManager;
pub use queries::*;
pub use query_params::*;
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;

mod attachment_data;
mod connection;
mod path_manager;
mod queries;
mod query_params;
mod utils;

pub struct Storage {
    config: Arc<Config>,
    path_manager: PathManager,
}

impl Storage {
    pub fn open(config: Arc<Config>) -> Result<Storage> {
        let path_manager = PathManager::new(config.arhiv_root.clone());
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(Storage {
            config,
            path_manager,
        })
    }

    pub fn create(prime: bool, config: Arc<Config>) -> Result<Storage> {
        let path_manager = PathManager::new(config.arhiv_root.clone());
        path_manager.create_dirs()?;

        let storage = Storage {
            config,
            path_manager,
        };

        let mut conn =
            MutStorageConnection::new(Connection::open(storage.path_manager.get_db_file())?);

        let tx = conn.get_tx()?;

        tx.create_tables()?;
        tx.set_setting("is_prime", Some(prime.to_string()))?;

        tx.commit()?;

        Ok(storage)
    }

    pub fn get_connection(&self) -> Result<StorageConnection> {
        let conn = Connection::open_with_flags(
            self.path_manager.get_db_file(),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;

        Ok(StorageConnection::new(conn))
    }

    pub fn get_writable_connection(&self) -> Result<MutStorageConnection> {
        let conn = Connection::open_with_flags(
            self.path_manager.get_db_file(),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;

        Ok(MutStorageConnection::new(conn))
    }

    pub fn get_attachment_data(&self, id: Id) -> AttachmentData {
        AttachmentData::new(id, &self.path_manager, &self.config)
    }
}
