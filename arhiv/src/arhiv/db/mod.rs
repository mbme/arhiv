use anyhow::*;
pub use connection::*;
pub use dto::*;
pub use queries::*;

use rusqlite::{Connection, OpenFlags};

mod connection;
mod dto;
mod queries;
mod utils;

pub struct DB {
    db_file: String,
}

impl DB {
    pub const VERSION: u8 = 1;

    pub fn open<S: Into<String>>(db_file: S) -> Result<DB> {
        Ok(DB {
            db_file: db_file.into(),
        })
    }

    pub fn create<S: Into<String>>(db_file: S) -> Result<DB> {
        let db_file = db_file.into();

        // create tables
        let conn = Connection::open(&db_file)?;
        // turn WAL only once as it's permanent pragma
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(DB { db_file })
    }

    pub fn get_connection(&self) -> Result<DBConnection> {
        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        Ok(DBConnection::new(conn))
    }

    pub fn get_writable_connection(&self) -> Result<MutDBConnection> {
        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_WRITE)?;

        Ok(MutDBConnection::new(conn))
    }
}
