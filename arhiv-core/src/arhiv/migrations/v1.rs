use anyhow::{ensure, Result};
use rusqlite::Connection;

use rs_utils::FsTransaction;

use super::migration::{get_rows_count, DBMigration};

pub struct MigrationV1;

impl DBMigration for MigrationV1 {
    fn get_version(&self) -> u8 {
        1
    }

    fn get_schema(&self) -> &str {
        include_str!("./v1.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO settings
                       SELECT * FROM old_db.settings;

            -- store schema version in settings
            INSERT INTO settings(key, value) VALUES ('schema_version', '0');

            -- remove db version from settings
            DELETE FROM settings WHERE key = 'db_version';

            INSERT INTO documents_snapshots
                        SELECT id, rev, prev_rev, type, created_at, updated_at, data
                        FROM old_db.documents_snapshots;

            -- erased documents must have prev_rev 0
            UPDATE documents_snapshots SET prev_rev = 0 WHERE type = '';
       ",
        )?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        let old_documents_snapshots_count = get_rows_count(conn, "old_db.documents_snapshots")?;
        let new_documents_snapshots_count = get_rows_count(conn, "documents_snapshots")?;

        ensure!(
            old_documents_snapshots_count == new_documents_snapshots_count,
            "snapshots count must stay the same"
        );

        let old_settings_count = get_rows_count(conn, "old_db.settings")?;
        let new_settings_count = get_rows_count(conn, "settings")?;

        ensure!(
            new_settings_count == old_settings_count,
            "settings count must stay the same"
        );

        Ok(())
    }
}
