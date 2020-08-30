use super::queries::*;
use anyhow::*;
use rusqlite::{Connection, Transaction};

pub struct StorageConnection {
    conn: Connection,
}

impl StorageConnection {
    pub fn new(conn: Connection) -> Self {
        StorageConnection { conn }
    }
}

pub struct MutStorageConnection {
    conn: Connection,
}

impl MutStorageConnection {
    pub fn new(conn: Connection) -> Self {
        MutStorageConnection { conn }
    }

    pub fn get_tx(&mut self) -> Result<TxStorageConnection> {
        let tx = self.conn.transaction()?;

        Ok(TxStorageConnection { tx })
    }
}

pub struct TxStorageConnection<'a> {
    tx: Transaction<'a>,
}

impl<'a> TxStorageConnection<'a> {
    pub fn commit(self) -> Result<()> {
        self.tx.commit()?;

        Ok(())
    }
}

impl Queries for StorageConnection {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl<'a> Queries for TxStorageConnection<'a> {
    fn get_connection(&self) -> &Connection {
        &self.tx
    }
}

impl<'a> MutableQueries for TxStorageConnection<'a> {}
