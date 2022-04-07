use anyhow::Result;
use rusqlite::Connection;

use rs_utils::FsTransaction;

use super::migration::{
    ensure_settings_count_stay_the_same, ensure_snapshots_count_stay_the_same, DBMigration,
};

pub struct MigrationV3;

impl DBMigration for MigrationV3 {
    fn get_version(&self) -> u8 {
        3
    }

    fn get_schema(&self) -> &str {
        include_str!("./v3.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO settings
                       SELECT * FROM old_db.settings;

            -- rename schema_version setting into data_version
            UPDATE settings SET key = 'data_version' WHERE key = 'schema_version';

            INSERT INTO documents_snapshots(id, rev, prev_rev, document_type, created_at, updated_at, data)
                       SELECT id, rev, prev_rev, type, created_at, updated_at, data FROM old_db.documents_snapshots;
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
