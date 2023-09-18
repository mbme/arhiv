use anyhow::{ensure, Context, Result};
use rusqlite::Connection;

use rs_utils::FsTransaction;

use crate::db_migrations::migration::get_rows_count;

use super::migration::{ensure_snapshots_count_stay_the_same, DBMigration};

pub struct MigrationV5;

impl DBMigration for MigrationV5 {
    fn get_version(&self) -> u8 {
        5
    }

    fn get_schema(&self) -> &str {
        include_str!("./v5.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO kvs(key, value)
                       SELECT * FROM old_db.kvs;",
        )?;

        let updated_rows = conn.execute("UPDATE kvs SET value = json_quote(value) WHERE key = json_array('settings', 'instance_id')", [])
            .context("failed to update settings.instance_id value type")?;
        ensure!(
            updated_rows == 1,
            "expected 1 row to be updated, got {updated_rows}"
        );

        let instance_id: String = conn
            .query_row(
                "SELECT value ->> '$' FROM kvs WHERE key = json_array('settings', 'instance_id')",
                [],
                |row| row.get(0),
            )
            .context("failed to update settings.instance_id value type")?;

        conn.execute_batch(&format!(
            "
            -- remove is_prime from settings
            DELETE FROM kvs WHERE key = json_array('settings', 'is_prime');

            -- drop created_at and prev_rev, update rev type
            INSERT INTO documents_snapshots(id, rev, document_type, subtype, updated_at, data)
                       SELECT
                         id,
                         CASE rev
                           WHEN 0 THEN 'null'
                           ELSE json_object('{instance_id}', rev)
                         END,
                         document_type,
                         subtype,
                         updated_at,
                         data
                       FROM old_db.documents_snapshots;
       "
        ))?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;

        let old_settings_count = get_rows_count(conn, "old_db.kvs")?;
        let new_settings_count = get_rows_count(conn, "kvs")?;

        ensure!(
            new_settings_count == old_settings_count - 1,
            "settings count must decrease by one"
        );

        Ok(())
    }
}
