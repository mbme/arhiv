use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;

use rs_utils::FsTransaction;

pub trait DBMigration {
    fn get_version(&self) -> u8;

    fn get_schema(&self) -> &str;

    fn apply(&self, conn: &Connection, fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()>;

    fn test(&self, conn: &Connection, data_dir: &str) -> Result<()>;
}

pub fn get_rows_count(conn: &Connection, table: &str) -> Result<u32> {
    conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
        row.get(0)
    })
    .context(anyhow!("failed to count number of rows in {}", table))
}
