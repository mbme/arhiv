use anyhow::Result;

use super::migration::Migration;

pub struct MigrationV1;

impl Migration for MigrationV1 {
    fn get_version(&self) -> u8 {
        1
    }

    fn get_schema(&self) -> &str {
        include_str!("./v1.sql")
    }

    fn apply(
        &self,
        conn: &rusqlite::Connection,
        _fs_tx: &mut rs_utils::FsTransaction,
        _data_dir: &str,
    ) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO settings
                       SELECT * FROM old_db.settings;

            -- store schema version in settings
            INSERT INTO settings(key, value) VALUES ('schema_version', '1');

            -- remove db version from settings
            DELETE FROM settings WHERE key = 'db_version';

            INSERT INTO documents_snapshots
                        SELECT id, rev, prev_rev, snapshot_id, type, created_at, updated_at, data
                        FROM old_db.documents_snapshots;

            -- erased documents must have prev_rev 0
            UPDATE documents_snapshots SET prev_rev = 0 WHERE type = '';
       ",
        )?;

        Ok(())
    }
}
