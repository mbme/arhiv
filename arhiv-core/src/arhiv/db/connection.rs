use std::sync::Arc;

use anyhow::{bail, ensure, Context, Result};
use fslock::LockFile;
use rusqlite::Connection;

use rs_utils::{log, FsTransaction};

use crate::{path_manager::PathManager, schema::DataSchema};

pub enum ArhivConnection {
    ReadOnly {
        conn: Connection,
        path_manager: Arc<PathManager>,
    },
    Transaction {
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,

        fs_tx: FsTransaction,
        lock_file: LockFile,

        completed: bool,
    },
}

impl ArhivConnection {
    pub fn new(conn: Connection, path_manager: Arc<PathManager>) -> Self {
        ArhivConnection::ReadOnly { conn, path_manager }
    }

    pub fn new_tx(
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,
    ) -> Result<Self> {
        conn.execute_batch("BEGIN DEFERRED")?;

        let lock_file = LockFile::open(&path_manager.lock_file)?;

        Ok(ArhivConnection::Transaction {
            conn,
            schema,
            completed: false,
            path_manager,
            fs_tx: FsTransaction::new(),
            lock_file,
        })
    }

    fn complete_tx(&mut self, commit: bool) -> Result<()> {
        match self {
            ArhivConnection::Transaction {
                completed,
                fs_tx,
                conn,
                ..
            } => {
                ensure!(!*completed, "transaction must not be completed");

                *completed = true;

                if commit {
                    fs_tx.commit()?;
                    conn.execute_batch("COMMIT")?;
                } else {
                    fs_tx.rollback()?;
                    conn.execute_batch("ROLLBACK")?;
                }
            }

            ArhivConnection::ReadOnly { .. } => bail!("not a transaction"),
        };

        Ok(())
    }

    pub fn commit(mut self) -> Result<()> {
        self.complete_tx(true)
    }

    pub fn rollback(&mut self) -> Result<()> {
        self.complete_tx(false)
    }

    pub(crate) fn get_schema(&self) -> Result<Arc<DataSchema>> {
        match self {
            ArhivConnection::Transaction { schema, .. } => Ok(schema.clone()),
            ArhivConnection::ReadOnly { .. } => bail!("not a transaction"),
        }
    }

    pub(crate) fn get_path_manager(&self) -> &PathManager {
        match self {
            ArhivConnection::ReadOnly { path_manager, .. }
            | ArhivConnection::Transaction { path_manager, .. } => path_manager,
        }
    }

    pub(crate) fn get_connection(&self) -> &Connection {
        match self {
            ArhivConnection::ReadOnly { conn, .. } | ArhivConnection::Transaction { conn, .. } => {
                conn
            }
        }
    }

    pub(crate) fn get_fs_tx(&mut self) -> Result<&mut FsTransaction> {
        match self {
            ArhivConnection::Transaction {
                lock_file,
                ref mut fs_tx,
                ..
            } => {
                if !lock_file.owns_lock() {
                    lock_file
                        .lock()
                        .context("failed to lock on arhiv lock file")?;
                }

                Ok(fs_tx)
            }
            ArhivConnection::ReadOnly { .. } => bail!("not a transaction"),
        }
    }
}

impl Drop for ArhivConnection {
    fn drop(&mut self) {
        match self {
            ArhivConnection::Transaction {
                lock_file,
                completed,
                ..
            } => {
                if lock_file.owns_lock() {
                    if let Err(err) = lock_file.unlock() {
                        log::error!("Failed to unlock arhiv lock file: {}", err);
                    }
                }

                if *completed {
                    return;
                }

                log::warn!("Transaction wasn't committed, rolling back");

                if let Err(err) = self.rollback() {
                    log::error!("Transaction rollback failed: {}", err);
                }
            }

            ArhivConnection::ReadOnly { .. } => {}
        };
    }
}
