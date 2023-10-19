use anyhow::{anyhow, ensure, Context, Result};
use rusqlite::Connection;

use rs_utils::FsTransaction;

pub trait DBMigration {
    fn get_version(&self) -> u8;

    fn get_schema(&self) -> &str;

    fn apply(&self, conn: &Connection, fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()>;

    fn test(&self, conn: &Connection, data_dir: &str) -> Result<()>;
}

pub fn get_rows_count(conn: &Connection, table: &str) -> Result<u32> {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .context(anyhow!("failed to count number of rows in {}", table))
}

pub fn ensure_snapshots_count_stay_the_same(conn: &Connection) -> Result<()> {
    let old_documents_snapshots_count = get_rows_count(conn, "old_db.documents_snapshots")?;
    let new_documents_snapshots_count = get_rows_count(conn, "documents_snapshots")?;

    ensure!(
        old_documents_snapshots_count == new_documents_snapshots_count,
        "snapshots count must stay the same"
    );

    Ok(())
}

pub fn ensure_settings_count_stay_the_same(conn: &Connection) -> Result<()> {
    let old_settings_count = get_rows_count(conn, "old_db.settings")?;
    let new_settings_count = get_rows_count(conn, "settings")?;

    ensure!(
        new_settings_count == old_settings_count,
        "settings count must stay the same"
    );

    Ok(())
}
