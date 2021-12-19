use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
};

use anyhow::*;
use rusqlite::{functions::FunctionFlags, Connection, OptionalExtension};
use serde_json::Value;

use rs_utils::{get_mime_type, log, FsTransaction, TempFile};

use crate::definitions::get_standard_schema;

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

    let schema = get_standard_schema();

    let mut db_version = {
        let db = DB::open(root_dir.clone(), schema.clone())?;
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

        let db = DB::open(root_dir.clone(), schema.clone())?;

        let temp_arhiv_dir =
            TempFile::new_with_details(format!("ArhivUpgrade{}-", upgrade_version), "");

        let mut fs_tx = FsTransaction::new();

        let new_db = DB::create(temp_arhiv_dir.as_ref().to_string(), schema.clone())?;

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
    upgrade_v6_to_v7,
    upgrade_v7_to_v8,
    upgrade_v8_to_v9,
    upgrade_v9_to_v10,
    upgrade_v10_to_v11,
    upgrade_v11_to_v12,
    upgrade_v12_to_v13,
];

// stub
#[allow(clippy::unnecessary_wraps)]
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
    fn update_data(ctx: &rusqlite::functions::Context) -> Result<Value> {
        let document_type = ctx.get_raw(0).as_str()?;

        let document_data = ctx.get_raw(1).as_str()?;
        let mut document_data: Value = serde_json::from_str(document_data)?;

        let (data_field, new_title_field) = match document_type {
            "note" => ("data", "title"),
            "project" => ("description", "name"),
            "task" => ("description", "title"),
            _ => bail!("unexpected document type {}", document_type),
        };

        let data = document_data
            .get(data_field)
            .ok_or_else(|| anyhow!("can't find field {} in {} data", data_field, document_type))?
            .as_str()
            .ok_or_else(|| {
                anyhow!(
                    "field {} in {} data isn't a string",
                    data_field,
                    document_type
                )
            })?;

        let mut lines_iter = data.trim_start().lines();

        let new_title = lines_iter
            .next()
            .unwrap_or("No title")
            .trim_start_matches('#')
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
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("Failed to read file name"))?;

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

/// in books, add "completed" and "collections"
fn upgrade_v6_to_v7(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '7' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;

        UPDATE documents_snapshots
                SET data = json_insert(
                    data,
                    '$.completed', json('true'),
                    '$.collections', json('[]')
                )
                WHERE type = 'book';
       ",
    )?;

    Ok(())
}

/// in films, rename "directors" to "creators"
fn upgrade_v7_to_v8(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '8' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;

        UPDATE documents_snapshots
                SET data = json_patch(
                    data,
                    json_object(
                        'creators', json_extract(data, '$.directors'),
                        'directors', null)
                )
                WHERE type = 'film';
       ",
    )?;

    Ok(())
}

/// in books, change "pages" type from string to number
fn upgrade_v8_to_v9(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '9' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT * FROM old_db.documents_snapshots;

        UPDATE documents_snapshots
                SET data = json_replace(
                    data,
                    '$.pages',
                    CAST(json_extract(data, '$.pages') AS INTEGER)
                )
                WHERE type = 'book' AND json_extract(data, '$.pages') IS NOT NULL;
       ",
    )?;

    Ok(())
}

/// remove `archived` and `refs` columns
fn upgrade_v9_to_v10(conn: &Connection, _fs_tx: &mut FsTransaction, _data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '10' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT id, rev, prev_rev, snapshot_id, type, created_at, updated_at, data
                       FROM old_db.documents_snapshots;
       ",
    )?;

    Ok(())
}

/// change document type for deleted documents from "tombstone" to "" (empty string)
fn upgrade_v10_to_v11(
    conn: &Connection,
    _fs_tx: &mut FsTransaction,
    _data_dir: &str,
) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '11' WHERE key = 'db_version';

        INSERT INTO documents_snapshots(id, rev, prev_rev, snapshot_id, type, created_at, updated_at, data)
                       SELECT id, rev, prev_rev, snapshot_id,
                        (CASE type WHEN 'tombstone' THEN '' ELSE type END),
                        created_at, updated_at, data
                       FROM old_db.documents_snapshots;
       ",
    )?;

    Ok(())
}

// use blake3 for blob hash, update attachment fields
// NOTE: all attachments data must exist locally
fn upgrade_v11_to_v12(conn: &Connection, fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '12' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT id, rev, prev_rev, snapshot_id, type, created_at, updated_at, data
                       FROM old_db.documents_snapshots;
       ",
    )?;

    // get a list of all attachments
    let attachment_ids: HashSet<String> = {
        let mut stmt = conn.prepare("SELECT id FROM old_db.documents WHERE type = 'attachment'")?;
        let mut rows = stmt.query([])?;

        let mut result = HashSet::new();
        while let Some(row) = rows.next()? {
            result.insert(row.get(0)?);
        }

        result
    };

    // rename blobs
    // keep a map of attachment_id => blob_id
    // keep a map of attachment_id => media_type
    let mut attachments: HashMap<String, String> = HashMap::new();
    let mut attachments_media_types: HashMap<String, String> = HashMap::new();
    for attachment_id in &attachment_ids {
        let file_path = format!("{}/{}", data_dir, attachment_id);
        let blob_id = BLOBId::from_file(&file_path)
            .context(format!("failed to calculate blob id for {}", &file_path))?;

        let media_type = get_mime_type(&file_path)?;

        fs_tx.move_file(
            file_path,
            format!("{}/{}", data_dir, blob_id.get_file_name()),
        )?;

        attachments.insert(attachment_id.clone(), blob_id.to_string());
        attachments_media_types.insert(attachment_id.clone(), media_type);
    }

    // iter through attachments
    // Update each attachment:
    // * remove data field sha256
    // * add data field blob with blob_id
    // * add data field media_type with media type

    let update_data = |ctx: &rusqlite::functions::Context| -> Result<Value> {
        let document_id = ctx.get_raw(0).as_str()?;

        let document_data = ctx.get_raw(1).as_str()?;
        let mut document_data: Value = serde_json::from_str(document_data)?;
        {
            let document_data = document_data.as_object_mut().unwrap();

            let media_type = attachments_media_types
                .get(document_id)
                .unwrap()
                .to_string();
            let blob_id = attachments.get(document_id).unwrap().to_string();

            document_data.remove("sha256");
            document_data.insert("blob".to_string(), blob_id.into());
            document_data.insert("media_type".to_string(), media_type.into());
        }

        Ok(document_data)
    };

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
                    SET data = update_data(id, data)
                    WHERE type = ?",
            ["attachment"],
        )
        .context("Failed to update documents")?;

    log::info!("Updated {} document snapshots", rows_updated);

    Ok(())
}

// add size to attachments
// NOTE: all attachments data must exist locally
fn upgrade_v12_to_v13(conn: &Connection, _fs_tx: &mut FsTransaction, data_dir: &str) -> Result<()> {
    conn.execute_batch(
        "INSERT INTO settings
                       SELECT * FROM old_db.settings;

        UPDATE settings SET value = '13' WHERE key = 'db_version';

        INSERT INTO documents_snapshots
                       SELECT id, rev, prev_rev, snapshot_id, type, created_at, updated_at, data
                       FROM old_db.documents_snapshots;
       ",
    )?;

    // iter through attachments
    //  add data field size to each attachment
    let update_data = |ctx: &rusqlite::functions::Context| -> Result<Value> {
        let document_data = ctx.get_raw(0).as_str()?;
        let mut document_data: Value = serde_json::from_str(document_data)?;
        {
            let document_data = document_data.as_object_mut().unwrap();
            let blob_id = document_data.get("blob").unwrap().as_str().unwrap();
            let blob_id = BLOBId::from_string(blob_id);

            let blob_file_path = format!("{}/{}", data_dir, blob_id.get_file_name());

            let size = fs::metadata(blob_file_path)?.len();

            document_data.insert("size".to_string(), size.into());
        }

        Ok(document_data)
    };

    conn.create_scalar_function(
        "update_data",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 1, "called with unexpected number of arguments");

            update_data(ctx).map_err(|e| rusqlite::Error::UserFunctionError(e.into()))
        },
    )
    .context(anyhow!("Failed to define update_data function"))?;

    let rows_updated = conn
        .execute(
            "UPDATE documents_snapshots
                    SET data = update_data(data)
                    WHERE type = ?",
            ["attachment"],
        )
        .context("Failed to update documents snapshots")?;

    log::info!("Updated {} document snapshots", rows_updated);

    Ok(())
}
