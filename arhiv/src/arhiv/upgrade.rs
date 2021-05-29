use anyhow::*;

use rs_utils::{log, FsTransaction, TempFile};
use rusqlite::Connection;

use super::db::*;

fn get_db_version(conn: &Connection) -> Result<u8> {
    let value: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'db_version'",
            [],
            |row| row.get(0),
        )
        .context("failed to read db_version")?;

    serde_json::from_str(&value).context("failed to parse db_version")
}

impl super::Arhiv {
    pub fn upgrade(root_dir: impl Into<String>) -> Result<()> {
        let root_dir = root_dir.into();

        let db_version = {
            let db = DB::open(root_dir.clone())?;
            let conn = db.open_connection(false)?;

            get_db_version(&conn)?
        };

        ensure!(
            db_version <= DB::VERSION,
            "DB version {} is greater than expected db version {}",
            db_version,
            DB::VERSION
        );

        ensure!(db_version > 0, "DB version must not be 0",);

        // check if upgrade is needed
        if db_version == DB::VERSION {
            log::info!(
                "DB version {} matches expected db version, nothing to upgrade",
                db_version
            );

            return Ok(());
        }

        log::info!(
            "DB version {}, starting upgrade to version {}",
            db_version,
            DB::VERSION
        );

        // while not latest version:
        //   create temp dir
        //   run upgrade(old db, new file)
        //   replace old db with new file
        //   remove temp dir
        for (pos, upgrade) in UPGRADES.iter().enumerate() {
            let upgrade_version: u8 = pos as u8 + 1;

            // skip irrelevant upgrades
            if db_version >= upgrade_version {
                continue;
            }

            log::debug!("upgrading db to version {}", upgrade_version);

            let db = DB::open(root_dir.clone())?;

            let temp_arhiv_dir =
                TempFile::new_with_details(format!("ArhivUpgrade{}-", upgrade_version), "");

            let new_db = DB::create(temp_arhiv_dir.as_ref().to_string())?;

            {
                let new_conn = new_db.open_connection(true)?;

                new_conn.execute_batch(&format!(
                    "ATTACH DATABASE '{}' AS 'old_db'",
                    db.get_db_file()
                ))?;

                new_conn.execute_batch("BEGIN DEFERRED")?;

                upgrade(&new_conn)?;

                new_conn.execute_batch("COMMIT")?;

                new_conn.execute("VACUUM", [])?;
            }

            let mut fs_tx = FsTransaction::new();
            fs_tx.move_file(new_db.get_db_file(), db.get_db_file())?;
            fs_tx.commit()?;

            log::info!("Upgraded db to version {}", upgrade_version);
        }

        log::info!("Done");

        Ok(())
    }
}

type Upgrade = fn(&Connection) -> Result<()>;

const UPGRADES: [Upgrade; DB::VERSION as usize] = [
    upgrade_v0_to_v1, //
    upgrade_v1_to_v2,
    upgrade_v2_to_v3,
];

// stub
fn upgrade_v0_to_v1(_conn: &Connection) -> Result<()> {
    Ok(())
}

fn upgrade_v1_to_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings(key, value)
                       SELECT key, value FROM old_db.settings;

        DELETE FROM settings WHERE key = 'schema_version';

        UPDATE settings SET value = '2' WHERE key = 'db_version';
       ",
    )
    .context("failed to migrate settings")?;

    // migrate documents from documents_history table
    {
        let rows_updated = conn.execute("INSERT INTO documents_snapshots(id, rev, prev_rev, snapshot_id, type, created_at, updated_at, archived, refs, data)
                                                 SELECT
                                                    id,
                                                    rev,
                                                    LEAD(rev, 1, 0) OVER (
                                                        PARTITION BY id
                                                        ORDER BY rev DESC
                                                        ROWS BETWEEN CURRENT ROW AND 1 FOLLOWING) prev_rev,
                                                    ABS(RANDOM()) snapshot_id,
                                                    type,
                                                    created_at,
                                                    updated_at,
                                                    archived,
                                                    refs,
                                                    data
                                                 FROM old_db.documents_history", [])
        .context("failed to migrate documents_history table")?;

        log::info!(
            "Migrated {} rows from documents_history table",
            rows_updated
        );
    }

    // copy modified documents from the documents table
    {
        let rows_updated = conn.execute("INSERT INTO documents_snapshots(id, rev, prev_rev, snapshot_id, type, created_at, updated_at, archived, refs, data)
                                                 SELECT
                                                    id,
                                                    rev,
                                                    (SELECT IFNULL(MAX(rev), 0) FROM documents_snapshots WHERE id = id) prev_rev,
                                                    ABS(RANDOM()) snapshot_id,
                                                    type,
                                                    created_at,
                                                    updated_at,
                                                    archived,
                                                    refs,
                                                    data
                                                FROM old_db.documents WHERE rev = 0", [])
        .context("failed to migrate documents table")?;

        log::info!("Migrated {} rows from documents table", rows_updated);
    }

    Ok(())
}

fn upgrade_v2_to_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '3' WHERE key = 'db_version';
       ",
    )
    .context("failed to migrate settings")?;

    conn.execute_batch(
        "INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;

        UPDATE documents_snapshots SET data = json_remove(data, '$.complexity') WHERE type = 'task';
       ",
    )
    .context("failed to migrate documents_snapshots table")?;

    Ok(())
}
