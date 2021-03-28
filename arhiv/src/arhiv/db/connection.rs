use super::queries::*;
use anyhow::*;
use rusqlite::{Connection, Transaction};

pub struct DBConnection {
    conn: Connection,
}

impl DBConnection {
    pub fn new(conn: Connection) -> Self {
        DBConnection { conn }
    }
}

pub struct MutDBConnection {
    conn: Connection,
}

impl MutDBConnection {
    pub fn new(conn: Connection) -> Self {
        MutDBConnection { conn }
    }

    pub fn get_tx(&mut self) -> Result<TxDBConnection> {
        let tx = self.conn.transaction()?;

        Ok(TxDBConnection { tx })
    }
}

pub struct TxDBConnection<'a> {
    tx: Transaction<'a>,
}

impl<'a> TxDBConnection<'a> {
    pub fn commit(self) -> Result<()> {
        self.tx.commit()?;

        Ok(())
    }
}

impl Queries for DBConnection {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl Queries for MutDBConnection {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl MutableQueries for MutDBConnection {}

impl<'a> Queries for TxDBConnection<'a> {
    fn get_connection(&self) -> &Connection {
        &self.tx
    }
}

impl<'a> MutableQueries for TxDBConnection<'a> {}
