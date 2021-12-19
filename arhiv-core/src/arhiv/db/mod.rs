use std::fs;
use std::sync::Arc;
use std::time::Instant;

use anyhow::*;
use rusqlite::functions::{Context as FunctionContext, FunctionFlags};
use rusqlite::{Connection, Error as RusqliteError, OpenFlags};

use rs_utils::log;
use serde_json::Value;

use crate::entities::{BLOBId, DocumentData};
use crate::schema::DataSchema;

pub use blob_queries::*;
pub use connection::*;
pub use dto::*;
pub use filter::*;
pub use migrations::apply_migrations;
use path_manager::PathManager;
pub use queries::*;

mod blob_queries;
mod connection;
mod dto;
mod filter;
mod migrations;
mod path_manager;
mod queries;
mod query_builder;
pub mod utils;

pub struct DB {
    path_manager: PathManager,
    schema: Arc<DataSchema>,
}

impl DB {
    pub const VERSION: u8 = 14;

    pub fn open(root_dir: String, schema: DataSchema) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(DB {
            path_manager,
            schema: Arc::new(schema),
        })
    }

    pub fn create(root_dir: String, schema: DataSchema) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.create_dirs()?;

        let db = DB {
            path_manager,
            schema: Arc::new(schema),
        };

        let conn = Connection::open_with_flags(
            &db.path_manager.db_file,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        init_extract_refs_fn(&conn, db.schema.clone())?;

        // turn WAL only once as it's permanent pragma
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // create tables
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(db)
    }

    pub(crate) fn get_db_file(&self) -> &str {
        &self.path_manager.db_file
    }

    pub(crate) fn open_connection(&self, mutable: bool) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            &self.path_manager.db_file,
            if mutable {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            },
        )?;

        init_extract_refs_fn(&conn, self.schema.clone())?;
        init_calculate_search_score_fn(&conn, self.schema.clone())?;
        init_json_contains(&conn)?;

        Ok(conn)
    }

    pub fn get_connection(&self) -> Result<ArhivConnection> {
        let conn = self.open_connection(false)?;

        Ok(ArhivConnection::new(conn, &self.path_manager))
    }

    pub fn get_tx(&self) -> Result<ArhivTransaction> {
        let conn = self.open_connection(true)?;

        ArhivTransaction::new(conn, &self.path_manager)
    }

    pub fn iter_blobs(&self) -> Result<impl Iterator<Item = Result<BLOBId>>> {
        Ok(
            fs::read_dir(&self.path_manager.data_dir)?.filter_map(|item| {
                let entry = match item {
                    Ok(entry) => entry,
                    Err(err) => return Some(Err(err).context("Failed to read data entry")),
                };

                let entry_path = entry.path();
                if entry_path.is_file() {
                    let file_name = entry_path
                        .file_name()
                        .ok_or_else(|| anyhow!("Failed to read file name"))
                        .map(|value| value.to_string_lossy().to_string())
                        .and_then(|value| BLOBId::from_file_name(&value));

                    return Some(file_name);
                }

                log::warn!("{} isn't a file", entry_path.to_string_lossy());

                None
            }),
        )
    }

    pub fn cleanup(&self) -> Result<()> {
        self.remove_orphaned_blobs()?;
        self.vacuum()?;

        Ok(())
    }

    fn vacuum(&self) -> Result<()> {
        let now = Instant::now();

        let conn = self.open_connection(true)?;
        conn.execute("VACUUM", [])?;

        log::debug!(
            "completed VACUUM in {} seconds",
            now.elapsed().as_secs_f32()
        );

        Ok(())
    }

    fn remove_orphaned_blobs(&self) -> Result<()> {
        let mut tx = self.get_tx()?;

        ensure!(
            !tx.has_staged_documents()?,
            "there must be no staged changes"
        );

        let used_blob_ids = tx.get_used_blob_ids()?;

        let mut removed_blobs = 0;
        for entry in self.iter_blobs()? {
            let id = entry?;

            if !used_blob_ids.contains(&id) {
                tx.remove_blob(&id)?;
                removed_blobs += 1;
            }
        }

        tx.commit()?;

        log::debug!("Removed {} orphaned blobs", removed_blobs);

        Ok(())
    }
}

fn json_contains(data: &str, field: &str, value: &str) -> Result<bool> {
    let data: Value = serde_json::from_str(data)?;

    let data = if let Some(data) = data.get(field) {
        data
    } else {
        return Ok(false);
    };

    if let Some(data) = data.as_str() {
        return Ok(data == value);
    }

    if let Some(data) = data.as_array() {
        let result = data
            .iter()
            .any(|item| item.as_str().map_or(false, |item| item == value));

        return Ok(result);
    }

    bail!("data must be string or array")
}

fn init_calculate_search_score_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let calculate_search_score = move |ctx: &FunctionContext| -> Result<usize> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let document_data = ctx
            .get_raw(1)
            .as_str()
            .context("document_data must be str")?;

        let pattern = ctx.get_raw(2).as_str().context("pattern must be str")?;

        if pattern.is_empty() {
            return Ok(1);
        }

        let data_description = schema.get_data_description(document_type)?;
        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let result = data_description.search(&document_data, pattern);

        if let Err(ref err) = result {
            log::error!("calculate_search_score() failed: \n{}", err);
        }

        result
    };

    conn.create_scalar_function(
        "calculate_search_score",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

            calculate_search_score(ctx)
                .context("calculate_search_score() failed")
                .map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'calculate_search_score'")
}

fn init_json_contains(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "json_contains",
        3,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

            let data = ctx.get_raw(0).as_str().expect("data must be str");
            let field = ctx.get_raw(1).as_str().expect("field must be str");
            let value = ctx.get_raw(2).as_str().expect("value must be str");

            json_contains(data, field, value)
                .context("json_contains() failed")
                .map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'json_contains'")
}

fn init_extract_refs_fn(conn: &Connection, schema: Arc<DataSchema>) -> Result<()> {
    // WARN: schema MUST be an Arc and MUST be moved into the closure in order for sqlite to work correctly

    let extract_refs = move |ctx: &FunctionContext| -> Result<String> {
        let document_type = ctx
            .get_raw(0)
            .as_str()
            .context("document_type must be str")?;

        let document_data = ctx
            .get_raw(1)
            .as_str()
            .context("document_data must be str")?;

        let document_data: DocumentData = serde_json::from_str(document_data)?;

        let refs = schema.extract_refs(document_type, &document_data)?;

        serde_json::to_string(&refs).context("failed to serialize refs")
    };

    conn.create_scalar_function(
        "extract_refs",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

            let result = extract_refs(ctx);

            if let Err(ref err) = result {
                log::error!("extract_refs() failed: \n{:?}", err);
            }

            result.map_err(|e| RusqliteError::UserFunctionError(e.into()))
        },
    )
    .context("Failed to define function 'extract_refs'")
}
