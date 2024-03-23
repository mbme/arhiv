use anyhow::{ensure, Result};
use rusqlite::Connection;

use rs_utils::FsTransaction;

use crate::db::migrations::migration::get_rows_count;

use super::migration::{ensure_snapshots_count_stay_the_same, DBMigration};

pub struct MigrationV6;

impl DBMigration for MigrationV6 {
    fn get_version(&self) -> u8 {
        6
    }

    fn get_schema(&self) -> &str {
        include_str!("./v6.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO kvs
                       SELECT * FROM old_db.kvs;

            INSERT INTO kvs(key, value) 
            VALUES 
                (json_array('settings', 'schema_name'), json_quote('arhiv')),
                (json_array('settings', 'login'), json_quote('arhiv')),
                (json_array('settings', 'password'), json_quote('arhiv12345678'));

            INSERT INTO documents_snapshots
                       SELECT id, rev, document_type, updated_at, data FROM old_db.documents_snapshots;
       ",
        )?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;

        let old_kvs_count = get_rows_count(conn, "old_db.kvs")?;
        let new_kvs_count = get_rows_count(conn, "kvs")?;

        ensure!(new_kvs_count == old_kvs_count + 3);

        Ok(())
    }
}
