use anyhow::*;
pub use connection::*;
pub use queries::*;
pub use query_params::*;
use rusqlite::{Connection, OpenFlags};
pub use settings::*;

mod connection;
mod queries;
mod query_params;
mod settings;
mod utils;

pub struct Storage {
    db_file: String,
}

impl Storage {
    pub fn open<S: Into<String>>(db_file: S) -> Result<Storage> {
        Ok(Storage {
            db_file: db_file.into(),
        })
    }

    pub fn create<S: Into<String>>(db_file: S) -> Result<Storage> {
        let db_file = db_file.into();

        let mut conn = MutStorageConnection::new(Connection::open(&db_file)?);

        let tx = conn.get_tx()?;

        tx.create_tables()?;

        tx.commit()?;

        Ok(Storage { db_file })
    }

    pub fn get_connection(&self) -> Result<StorageConnection> {
        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        Ok(StorageConnection::new(conn))
    }

    pub fn get_writable_connection(&self) -> Result<MutStorageConnection> {
        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_WRITE)?;

        Ok(MutStorageConnection::new(conn))
    }
}
