use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, ensure, Context, Result};
use rusqlite::{
    functions::{Context as FunctionContext, FunctionFlags},
    Connection, Error as RusqliteError, OpenFlags,
};
use serde_json::Value;

use rs_utils::log;

use crate::{entities::DocumentData, path_manager::PathManager, schema::DataSchema};

pub use blob_queries::*;
pub use connection::*;
pub use dto::*;
pub use filter::*;
pub use queries::*;

mod blob_queries;
mod connection;
mod dto;
mod filter;
mod queries;
mod query_builder;
pub mod utils;

pub struct DB {
    pub(super) path_manager: PathManager,
    pub(super) schema: Arc<DataSchema>,
}

impl DB {
    pub fn open(root_dir: String, schema: DataSchema) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(DB {
            path_manager,
            schema: Arc::new(schema),
        })
    }

    fn open_connection(&self, mutable: bool) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            &self.path_manager.db_file,
            if mutable {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            },
        )
        .context("failed to open connection")?;

        conn.pragma_update(None, "foreign_keys", true)
            .context("failed to enable foreign keys support")?;

        init_extract_refs_fn(&conn, self.schema.clone())?;
        init_calculate_search_score_fn(&conn, self.schema.clone())?;
        init_json_contains(&conn)?;

        Ok(conn)
    }

    pub fn get_connection(&self) -> Result<ArhivConnection> {
        let conn = self.open_connection(false)?;

        Ok(ArhivConnection::new(
            conn,
            self.path_manager.data_dir.clone(),
        ))
    }

    pub fn get_tx(&self) -> Result<ArhivTransaction> {
        let conn = self.open_connection(true)?;

        ArhivTransaction::new(
            conn,
            self.path_manager.data_dir.clone(),
            self.schema.clone(),
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
        for blob_id in tx.list_local_blobs()? {
            if !used_blob_ids.contains(&blob_id) {
                tx.remove_blob(&blob_id)?;
                removed_blobs += 1;
            }
        }

        tx.commit()?;

        log::debug!("Removed {} orphaned blobs", removed_blobs);

        Ok(())
    }

    pub(super) fn compute_data(&self) -> Result<()> {
        let now = Instant::now();

        let conn = self.open_connection(true)?;

        let rows_count = conn.execute(
            "INSERT INTO documents_refs(id, rev, refs)
               SELECT id, rev, extract_refs(type, data)
               FROM documents_snapshots ds
               WHERE NOT EXISTS (
                 SELECT 1 FROM documents_refs dr WHERE dr.id = ds.id AND dr.rev = ds.rev
               )",
            [],
        )?;

        if rows_count > 0 {
            log::info!(
                "computed {} rows in {} seconds",
                rows_count,
                now.elapsed().as_secs_f32()
            );
        }

        Ok(())
    }

    pub fn apply_migrations(&self) -> Result<()> {
        let schema_version = self.get_connection()?.get_setting(SETTING_SCHEMA_VERSION)?;

        let migrations: Vec<_> = self
            .schema
            .get_migrations()
            .iter()
            .filter(|migration| migration.get_version() > schema_version)
            .cloned()
            .collect();

        if migrations.is_empty() {
            log::debug!("no schema migrations to apply");

            return Ok(());
        }

        log::info!("{} schema migrations to apply", migrations.len());

        let new_schema_version = self.schema.get_version();

        let tx = self.get_tx()?;

        let migrations = Arc::new(migrations);
        let apply_migrations = move |ctx: &FunctionContext| -> Result<String> {
            let document_type = ctx
                .get_raw(0)
                .as_str()
                .context("document_type must be str")?;

            let document_data = ctx
                .get_raw(1)
                .as_str()
                .context("document_data must be str")?;

            let mut document_data: DocumentData = serde_json::from_str(document_data)?;

            for migration in migrations.as_ref() {
                migration.update(document_type, &mut document_data)?;
            }

            serde_json::to_string(&document_data).context("failed to serialize document_data")
        };

        tx.conn
            .create_scalar_function(
                "apply_migrations",
                2,
                FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
                move |ctx| {
                    assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

                    let result = apply_migrations(ctx);

                    if let Err(ref err) = result {
                        log::error!("apply_migrations() failed: \n{:?}", err);
                    }

                    result.map_err(|e| RusqliteError::UserFunctionError(e.into()))
                },
            )
            .context("Failed to define function 'apply_migrations'")?;

        tx.apply_migrations()?;
        tx.set_setting(SETTING_SCHEMA_VERSION, new_schema_version)?;

        tx.commit()?;

        log::info!(
            "Finished schema migration from version {} to {}",
            schema_version,
            new_schema_version
        );

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
