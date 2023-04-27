use anyhow::Result;
use rusqlite::Connection;

use rs_utils::{generate_random_id, FsTransaction};

use super::migration::{
    ensure_settings_count_stay_the_same, ensure_snapshots_count_stay_the_same, DBMigration,
};

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
            "INSERT INTO settings
                       SELECT * FROM old_db.settings;

            -- remove arhiv_id from settings
            DELETE FROM settings WHERE key = 'arhiv_id';

            INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;
       ",
        )?;

        // generate random instance id
        conn.execute(
            "INSERT INTO settings(key, value) VALUES('instance_id', ?1)",
            [instance_id],
        )?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;
        ensure_settings_count_stay_the_same(conn)?;

        Ok(())
    }
}
