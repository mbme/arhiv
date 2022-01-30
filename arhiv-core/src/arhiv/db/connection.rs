use anyhow::{ensure, Result};
use rusqlite::Connection;

use rs_utils::{log, FsTransaction};

use super::{blob_queries::*, queries::*};

pub struct ArhivConnection {
    conn: Connection,
    data_dir: String,
}

impl<'a> ArhivConnection {
    pub fn new(conn: Connection, data_dir: String) -> Self {
        ArhivConnection { conn, data_dir }
    }
}

pub struct ArhivTransaction {
    pub(crate) conn: Connection,
    finished: bool,

    data_dir: String,
    fs_tx: FsTransaction,
}

impl ArhivTransaction {
    pub fn new(conn: Connection, data_dir: String) -> Result<Self> {
        conn.execute_batch("BEGIN DEFERRED")?;

        Ok(ArhivTransaction {
            conn,
            finished: false,
            data_dir,
            fs_tx: FsTransaction::new(),
        })
    }

    pub fn commit(mut self) -> Result<()> {
        ensure!(
            !self.finished,
            "must not try to commit finished transaction"
        );

        self.finished = true;

        self.fs_tx.commit()?;
        self.conn.execute_batch("COMMIT")?;

        Ok(())
    }

    pub fn rollback(&mut self) -> Result<()> {
        ensure!(
            !self.finished,
            "must not try to rollback finished transaction"
        );

        self.finished = true;

        self.conn.execute_batch("ROLLBACK")?;

        Ok(())
    }
}

impl Drop for ArhivTransaction {
    fn drop(&mut self) {
        if self.finished {
            return;
        }

        log::warn!("Transaction wasn't committed, rolling back");

        if let Err(err) = self.rollback() {
            log::error!("Transaction rollback failed: {}", err);
        }
    }
}

impl Queries for ArhivConnection {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl BLOBQueries for ArhivConnection {
    fn get_data_dir(&self) -> &str {
        &self.data_dir
    }
}

impl Queries for ArhivTransaction {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl BLOBQueries for ArhivTransaction {
    fn get_data_dir(&self) -> &str {
        &self.data_dir
    }
}

impl MutableQueries for ArhivTransaction {}

impl MutableBLOBQueries for ArhivTransaction {
    fn get_fs_tx(&mut self) -> &mut FsTransaction {
        &mut self.fs_tx
    }
}
