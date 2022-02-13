use anyhow::Result;
use rusqlite::Connection;

use rs_utils::FsTransaction;

use super::migration::{
    ensure_settings_count_stay_the_same, ensure_snapshots_count_stay_the_same, DBMigration,
};

pub struct MigrationV2;

impl DBMigration for MigrationV2 {
    fn get_version(&self) -> u8 {
        2
    }

    fn get_schema(&self) -> &str {
        include_str!("./v2.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO settings
                       SELECT * FROM old_db.settings;

            INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;
       ",
        )?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;
        ensure_settings_count_stay_the_same(conn)?;

        Ok(())
    }
}
