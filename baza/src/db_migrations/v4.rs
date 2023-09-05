use anyhow::{ensure, Result};
use rusqlite::Connection;

use rs_utils::{generate_random_id, FsTransaction};

use crate::db_migrations::migration::get_rows_count;

use super::migration::{ensure_snapshots_count_stay_the_same, DBMigration};

pub struct MigrationV4;

impl DBMigration for MigrationV4 {
    fn get_version(&self) -> u8 {
        4
    }

    fn get_schema(&self) -> &str {
        include_str!("./v4.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        let instance_id = generate_random_id();
        conn.execute_batch(
            "INSERT INTO kvs(key, value)
                       SELECT * FROM old_db.settings;

            -- remove arhiv_id from settings
            DELETE FROM kvs WHERE key = 'arhiv_id';

            -- remove is_prime from settings
            DELETE FROM kvs WHERE key = 'is_prime';

            -- convert keys into kvs key array
            UPDATE kvs SET key = json_array('settings', key);

            INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;
       ",
        )?;

        // generate random instance id
        conn.execute(
            "INSERT INTO kvs(key, value) VALUES(json_array('settings', 'instance_id'), ?1)",
            [instance_id],
        )?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;

        let old_settings_count = get_rows_count(conn, "old_db.settings")?;
        let new_settings_count = get_rows_count(conn, "kvs")?;

        ensure!(
            new_settings_count == old_settings_count,
            "settings count must stay the same"
        );

        Ok(())
    }
}
