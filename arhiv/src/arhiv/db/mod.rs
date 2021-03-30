use anyhow::*;
pub use connection::*;
pub use dto::*;
pub use queries::*;
use rs_utils::fuzzy_match;
use rusqlite::{functions::FunctionFlags, Connection, Error as RusqliteError, OpenFlags};

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
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(DB { db_file })
    }

    pub fn get_connection(&self) -> Result<DBConnection> {
        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        self.init_pragmas(&conn)?;
        self.init_fuzzy_search(&conn)?;

        Ok(DBConnection::new(conn))
    }

    pub fn get_writable_connection(&self) -> Result<MutDBConnection> {
        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_WRITE)?;

        self.init_pragmas(&conn)?;
        self.init_fuzzy_search(&conn)?;

        Ok(MutDBConnection::new(conn))
    }

    fn init_pragmas(&self, conn: &Connection) -> Result<()> {
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        Ok(())
    }

    fn init_fuzzy_search(&self, conn: &Connection) -> Result<()> {
        conn.create_scalar_function(
            "fuzzySearch",
            2,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
            move |ctx| {
                assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

                let haystack = ctx
                    .get_raw(0)
                    .as_str()
                    .map_err(|e| RusqliteError::UserFunctionError(e.into()))?;

                let needle = ctx
                    .get_raw(1)
                    .as_str()
                    .map_err(|e| RusqliteError::UserFunctionError(e.into()))?;

                Ok(fuzzy_match(needle, haystack))
            },
        )
        .context(anyhow!("Failed to define fuzzySearch function"))
    }
}
