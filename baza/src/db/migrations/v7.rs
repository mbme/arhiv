use std::{collections::HashMap, fs};

use anyhow::{bail, ensure, Result};
use rusqlite::Connection;

use rs_utils::{path_to_string, FsTransaction};

use crate::entities::BLOBId;

use super::migration::{
    ensure_kvs_count_stay_the_same, ensure_snapshots_count_stay_the_same, DBMigration,
};

pub struct MigrationV7;

impl DBMigration for MigrationV7 {
    fn get_version(&self) -> u8 {
        7
    }

    fn get_schema(&self) -> &str {
        include_str!("./v7.sql")
    }

    fn apply(&self, conn: &Connection, _fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()> {
        conn.execute_batch(
            "INSERT INTO kvs
                       SELECT * FROM old_db.kvs;

            INSERT INTO documents_snapshots
                       SELECT id, rev, document_type, updated_at, data FROM old_db.documents_snapshots;
       ",
        )?;

        let mut blob_ids_map = HashMap::new();

        for entry in fs::read_dir(data_dir)? {
            let entry = entry?;
            let path = entry.path();
            ensure!(
                path.is_file(),
                "Expected only files in data_dir, got {path:?}"
            );

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("blake3-") {
                    let new_blob_id = BLOBId::from_file(&path_to_string(&path))?;

                    let new_file_name = new_blob_id.to_string();
                    let new_path = path.with_file_name(new_file_name);

                    fs::rename(&path, &new_path)?;

                    let blake3_hash = file_name.trim_start_matches("blake3-");
                    blob_ids_map.insert(blake3_hash.to_string(), new_blob_id.to_string());
                } else {
                    bail!("Unexpected file {path:?} in data_dir");
                }
            }
        }

        for (old_blob_id, new_blob_id) in blob_ids_map {
            conn.execute(
                "UPDATE documents_snapshots SET data = json_set(data, '$.blob', ?) WHERE json_extract(data, '$.blob') = ?",
                [&new_blob_id, &old_blob_id],
            )?;
        }

        Ok(())
    }

    fn test(&self, conn: &Connection, _data_dir: &str) -> Result<()> {
        ensure_snapshots_count_stay_the_same(conn)?;
        ensure_kvs_count_stay_the_same(conn)?;

        Ok(())
    }
}
