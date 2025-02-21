use anyhow::Result;
use rusqlite::Connection;

use rs_utils::FsTransaction;

use super::migration::{
    ensure_kvs_count_stay_the_same, ensure_snapshots_count_stay_the_same, DBMigration,
};

pub struct MigrationV8;

impl DBMigration for MigrationV8 {
    fn get_version(&self) -> u8 {
        8
    }

    fn get_schema(&self) -> &str {
        include_str!("./v8.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO kvs
                       SELECT * FROM old_db.kvs;

            INSERT INTO documents_snapshots
                       SELECT id, rev, document_type, updated_at, data FROM old_db.documents_snapshots;
       ",
        )?;

        conn.execute(
            "UPDATE documents_snapshots SET document_type = 'asset' WHERE document_type = 'attachment'",
            [],
        )?;

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;
        ensure_kvs_count_stay_the_same(conn)?;

        Ok(())
    }
}
