use std::fs;
use std::sync::Arc;
use std::time::Instant;

use anyhow::*;
use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, Error as RusqliteError, OpenFlags};

use rs_utils::log;
use serde_json::Value;

use crate::entities::Id;
use crate::schema::DataSchema;

pub use attachment_data::AttachmentData;
pub use blob_queries::*;
pub use connection::*;
pub use dto::*;
pub use filter::*;
pub use migrations::apply_migrations;
use path_manager::PathManager;
pub use queries::*;

mod attachment_data;
mod blob_queries;
mod connection;
mod dto;
mod filter;
mod migrations;
mod path_manager;
mod queries;
mod query_builder;
pub mod utils;

enum DocumentSearch {
    SimpleSearch,
    SchemaSearch(DataSchema),
}

impl DocumentSearch {
    fn search(&self, document_type: &str, document_data: &str, pattern: &str) -> Result<usize> {
        match self {
            Self::SimpleSearch => {
                let score = if document_data.contains(pattern) {
                    1
                } else {
                    0
                };

                Ok(score)
            }

            Self::SchemaSearch(schema) => {
                let data_description = schema.get_data_description(document_type)?;
                let data: Value = serde_json::from_str(document_data)?;

                data_description.search(&data, pattern)
            }
        }
    }
}

pub struct DB {
    path_manager: PathManager,
    document_search: Arc<DocumentSearch>,
}

impl DB {
    pub const VERSION: u8 = 10;

    pub fn open(root_dir: String) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(DB {
            path_manager,
            document_search: Arc::new(DocumentSearch::SimpleSearch),
        })
    }

    pub fn create(root_dir: String) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.create_dirs()?;

        let conn = Connection::open(&path_manager.db_file)?;

        // turn WAL only once as it's permanent pragma
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // create tables
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(DB {
            path_manager,
            document_search: Arc::new(DocumentSearch::SimpleSearch),
        })
    }

    pub fn with_schema_search(&mut self, schema: DataSchema) {
        self.document_search = Arc::new(DocumentSearch::SchemaSearch(schema));
    }

    pub fn get_db_file(&self) -> &str {
        &self.path_manager.db_file
    }

    pub fn open_connection(&self, mutable: bool) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            &self.path_manager.db_file,
            if mutable {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            },
        )?;

        Ok(conn)
    }

    pub fn get_connection(&self) -> Result<ArhivConnection> {
        let conn = self.open_connection(false)?;

        self.init_calculate_search_score_fn(&conn)?;
        self.init_json_contains(&conn)?;

        Ok(ArhivConnection::new(conn, &self.path_manager))
    }

    pub fn get_tx(&self) -> Result<ArhivTransaction> {
        let conn = self.open_connection(true)?;

        ArhivTransaction::new(conn, &self.path_manager)
    }

    fn init_calculate_search_score_fn(&self, conn: &Connection) -> Result<()> {
        let document_search = self.document_search.clone();

        conn.create_scalar_function(
            "calculate_search_score",
            3,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
            move |ctx| {
                assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

                let document_type = ctx.get_raw(0).as_str().expect("document_type must be str");

                let document_data = ctx.get_raw(1).as_str().expect("document_data must be str");

                let pattern = ctx.get_raw(2).as_str().expect("pattern must be str");

                if pattern.is_empty() {
                    return Ok(1);
                }

                let result = document_search.search(document_type, document_data, pattern);

                if let Err(ref err) = result {
                    log::error!("calculate_search_score() failed: \n{}", err);
                }

                result
                    .context("calculate_search_score() failed")
                    .map_err(|e| RusqliteError::UserFunctionError(e.into()))
            },
        )
        .context(anyhow!(
            "Failed to define function 'calculate_search_score'"
        ))
    }

    #[allow(clippy::unused_self)]
    fn init_json_contains(&self, conn: &Connection) -> Result<()> {
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
        .context(anyhow!("Failed to define function 'json_contains'"))
    }

    pub fn iter_blobs(&self) -> Result<impl Iterator<Item = Result<Id>>> {
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
                        .map(|value| value.to_string_lossy().to_string().into());

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

        let attachment_ids = tx.get_attachment_ids()?;

        let mut removed_blobs = 0;
        for entry in self.iter_blobs()? {
            let id = entry?;

            if !attachment_ids.contains(&id) {
                tx.remove_attachment_data(&id)?;
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
