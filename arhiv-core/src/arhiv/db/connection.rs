use anyhow::*;
use rusqlite::Connection;

use super::{blob_queries::*, path_manager::PathManager, queries::*};
use rs_utils::FsTransaction;

pub struct ArhivConnection<'a> {
    conn: Connection,
    path_manager: &'a PathManager,
}

impl<'a> ArhivConnection<'a> {
    pub fn new(conn: Connection, path_manager: &'a PathManager) -> Self {
        ArhivConnection { conn, path_manager }
    }
}

pub struct ArhivTransaction<'a> {
    conn: Connection,
    finished: bool,

    path_manager: &'a PathManager,
    fs_tx: FsTransaction,
}

impl<'a> ArhivTransaction<'a> {
    pub fn new(conn: Connection, path_manager: &'a PathManager) -> Result<Self> {
        conn.execute_batch("BEGIN DEFERRED")?;

        Ok(ArhivTransaction {
            conn,
            finished: false,
            path_manager,
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

#[allow(unused_must_use)]
impl<'a> Drop for ArhivTransaction<'a> {
    fn drop(&mut self) {
        if !self.finished {
            self.rollback();
        }
    }
}

impl<'a> Queries for ArhivConnection<'a> {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl<'a> BLOBQueries for ArhivConnection<'a> {
    fn get_data_dir(&self) -> &str {
        &self.path_manager.data_dir
    }
}

impl<'a> Queries for ArhivTransaction<'a> {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl<'a> BLOBQueries for ArhivTransaction<'a> {
    fn get_data_dir(&self) -> &str {
        &self.path_manager.data_dir
    }
}

impl<'a> MutableQueries for ArhivTransaction<'a> {}

impl<'a> MutableBLOBQueries for ArhivTransaction<'a> {
    fn get_fs_tx(&mut self) -> &mut FsTransaction {
        &mut self.fs_tx
    }
}
