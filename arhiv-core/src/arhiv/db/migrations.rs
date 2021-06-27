use anyhow::*;

use rs_utils::{log, FsTransaction, TempFile};
use rusqlite::{functions::FunctionFlags, Connection, OptionalExtension};
use serde_json::Value;

use super::*;

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

pub fn apply_migrations(root_dir: impl Into<String>) -> Result<()> {
    let root_dir = root_dir.into();

    let mut db_version = {
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

        log::info!("Upgrading db to version {}...", upgrade_version);

        let db = DB::open(root_dir.clone())?;

        let temp_arhiv_dir =
            TempFile::new_with_details(format!("ArhivUpgrade{}-", upgrade_version), "");

        let mut fs_tx = FsTransaction::new();

        let new_db = DB::create(temp_arhiv_dir.as_ref().to_string())?;

        {
            let new_conn = new_db.open_connection(true)?;

            new_conn.execute_batch(&format!(
                "ATTACH DATABASE '{}' AS 'old_db'",
                db.get_db_file()
            ))?;

            new_conn.execute_batch("BEGIN DEFERRED")?;

            upgrade(&new_conn, &mut fs_tx, &db.path_manager.data_dir)?;

            new_conn.execute_batch("COMMIT")?;

            new_conn.execute("VACUUM", [])?;
        }

        fs_tx.move_file(new_db.get_db_file(), db.get_db_file())?;
        fs_tx.commit()?;

        log::info!(
            "Upgraded db from version {} to version {}",
            db_version,
            upgrade_version
        );

        db_version = upgrade_version;
    }

    log::info!("Done");

    Ok(())
}

type Upgrade = fn(&Connection, &mut FsTransaction, &str) -> Result<()>;

const UPGRADES: [Upgrade; DB::VERSION as usize] = [
    upgrade_v0_to_v1, //
    upgrade_v1_to_v2,
    upgrade_v2_to_v3,
    upgrade_v3_to_v4,
    upgrade_v4_to_v5,
    upgrade_v5_to_v6,
];

// stub
fn upgrade_v0_to_v1(_conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    Ok(())
}

fn upgrade_v1_to_v2(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
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

fn upgrade_v2_to_v3(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
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

fn upgrade_v3_to_v4(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '4' WHERE key = 'db_version';
        INSERT INTO settings(key, value) VALUES ('schema_version', '1');
       ",
    )
    .context("failed to migrate settings")?;

    conn.execute_batch(
        "INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots",
    )
    .context("failed to migrate documents_snapshots table")?;

    Ok(())
}

fn upgrade_v4_to_v5(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        DELETE FROM settings WHERE key = 'schema_version';

        UPDATE settings SET value = '5' WHERE key = 'db_version';
       ",
    )
    .context("failed to migrate settings")?;

    conn.execute_batch(
        "INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots",
    )
    .context("failed to migrate documents_snapshots table")?;

    fn update_data(ctx: &rusqlite::functions::Context) -> Result<Value> {
        let document_type = ctx.get_raw(0).as_str()?;

        let document_data = ctx.get_raw(1).as_str()?;
        let mut document_data: Value = serde_json::from_str(&document_data)?;

        let (data_field, new_title_field) = match document_type {
            "note" => ("data", "title"),
            "project" => ("description", "name"),
            "task" => ("description", "title"),
            _ => bail!("unexpected document type {}", document_type),
        };

        let data = document_data
            .get(data_field)
            .ok_or(anyhow!(
                "can't find field {} in {} data",
                data_field,
                document_type
            ))?
            .as_str()
            .ok_or(anyhow!(
                "field {} in {} data isn't a string",
                data_field,
                document_type
            ))?;

        if data_field == "project" {
            println!("data:\n{}", data);
        }

        let mut lines_iter = data.trim_start().lines().into_iter();

        let new_title = lines_iter
            .next()
            .unwrap_or("No title")
            .trim_start_matches("#")
            .trim()
            .to_string();

        let new_data = lines_iter.collect::<Vec<_>>().join("\n");
        let new_data = new_data.trim();

        {
            let document_data = document_data.as_object_mut().unwrap();

            document_data.insert(new_title_field.to_string(), new_title.into());
            document_data.insert(data_field.to_string(), new_data.into());
        }

        Ok(document_data)
    }

    conn.create_scalar_function(
        "update_data",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

            update_data(ctx).map_err(|e| rusqlite::Error::UserFunctionError(e.into()))
        },
    )
    .context(anyhow!("Failed to define update_data function"))?;

    let rows_updated = conn
        .execute(
            "UPDATE documents_snapshots
                    SET data = update_data(type, data)
                    WHERE type = ?
                        OR type = ?
                        OR type = ?",
            ["note", "project", "task"],
        )
        .context("Failed to update documents")?;

    log::info!("Updated {} document snapshots", rows_updated);

    Ok(())
}

fn upgrade_v5_to_v6(conn: &Connection, fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '6' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;

        UPDATE documents_snapshots
                SET data = json_object(
                    'filename', json_extract(data, '$.filename'),
                    'sha256', json_extract(data, '$.hash')
                )
                WHERE type = 'attachment';
       ",
    )?;

    // rename blobs, use attachment id instead of sha256 blob hash
    let mut blobs_renamed = 0;
    for item in fs::read_dir(data_dir)? {
        let path = item?.path();

        if !path.is_file() {
            log::warn!(
                "migrating attachments: {} is not a file, skipping",
                path.to_str().unwrap_or("")
            );
            continue;
        }

        // read file name, which is a sha256 hash atm.
        let hash = path
            .file_name()
            .map(|filename| filename.to_str())
            .flatten()
            .ok_or(anyhow!("Failed to read file name"))?;

        // find attachment id by hash
        let id: Option<String> = conn
            .query_row(
                "SELECT id FROM documents_snapshots 
                    WHERE type = 'attachment' AND json_extract(data, '$.sha256') = ?",
                [hash],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(id) = id {
            // use attachment id as file name
            fs_tx.move_file(
                format!("{}/{}", data_dir, hash),
                format!("{}/{}", data_dir, id),
            )?;
            blobs_renamed += 1;
        } else {
            log::warn!(
                "migrating attachments: can't find attachment which owns {}, skipping",
                hash
            );
        }
    }

    log::info!("migrating attachments: renamed {} blobs", blobs_renamed);

    Ok(())
}
