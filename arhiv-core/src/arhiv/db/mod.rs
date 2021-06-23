use std::fs;
use std::time::Instant;

use anyhow::*;
use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, Error as RusqliteError, OpenFlags};

use rs_utils::log;
use utils::multi_search;

use crate::entities::BLOBHash;
use crate::schema::SCHEMA;

pub use attachment_data::AttachmentData;
pub use blob::*;
pub use connection::*;
pub use dto::*;
use path_manager::PathManager;
pub use queries::*;

mod attachment_data;
mod blob;
mod connection;
mod dto;
mod path_manager;
mod queries;
mod query_builder;
mod utils;

pub struct DB {
    path_manager: PathManager,
}

impl DB {
    pub const VERSION: u8 = 4;

    pub fn open(root_dir: String) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(DB { path_manager })
    }

    pub fn create(root_dir: String) -> Result<DB> {
        let path_manager = PathManager::new(root_dir);
        path_manager.create_dirs()?;

        let conn = Connection::open(&path_manager.db_file)?;

        // turn WAL only once as it's permanent pragma
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // create tables
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(DB { path_manager })
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

        Ok(ArhivConnection::new(conn, &self.path_manager))
    }

    pub fn get_tx(&self) -> Result<ArhivTransaction> {
        let conn = self.open_connection(true)?;

        ArhivTransaction::new(conn, &self.path_manager)
    }

    fn init_calculate_search_score_fn(&self, conn: &Connection) -> Result<()> {
        conn.create_scalar_function(
            "calculate_search_score",
            3,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
            move |ctx| {
                assert_eq!(ctx.len(), 3, "called with unexpected number of arguments");

                calculate_search_score(ctx).map_err(|e| RusqliteError::UserFunctionError(e.into()))
            },
        )
        .context(anyhow!("Failed to define calculate_search_score function"))
    }

    pub fn iter_blobs(&self) -> Result<impl Iterator<Item = Result<BLOBHash>>> {
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
                        .ok_or(anyhow!("Failed to read file name"))
                        .map(|value| BLOBHash::from_string(value.to_string_lossy().to_string()));

                    return Some(file_name);
                } else {
                    log::warn!("{} isn't a file", entry_path.to_string_lossy());

                    return None;
                }
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
        let hashes = tx.get_blob_hashes()?;

        let mut removed_blobs = 0;
        for entry in self.iter_blobs()? {
            let hash = entry?;

            if !hashes.contains(&hash) {
                tx.remove_attachment_data(&hash)?;
                removed_blobs += 1;
            }
        }

        tx.commit()?;

        log::debug!("Removed {} orphaned blobs", removed_blobs);

        Ok(())
    }
}

fn calculate_search_score(ctx: &rusqlite::functions::Context) -> Result<u32> {
    let document_type = ctx.get_raw(0).as_str()?;

    let document_data = ctx.get_raw(1).as_str()?;

    let pattern = ctx.get_raw(2).as_str()?;

    if pattern.is_empty() {
        return Ok(1);
    }

    let data = SCHEMA.extract_search_data(document_type, document_data)?;

    Ok(multi_search(pattern, &data))
}
