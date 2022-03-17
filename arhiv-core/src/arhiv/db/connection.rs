use std::{sync::Arc, time::Instant};

use anyhow::{anyhow, bail, ensure, Context, Result};
use chrono::Utc;
use fslock::LockFile;
use rusqlite::{
    functions::{Context as FunctionContext, FunctionFlags},
    Connection, Error as RusqliteError,
};

use rs_utils::{log, FsTransaction};

use crate::{
    db::SETTING_SCHEMA_VERSION,
    entities::{Document, DocumentData, Id, Revision},
    path_manager::PathManager,
    schema::DataSchema,
    Validator,
};

use super::{blob_queries::*, queries::*};

pub enum ArhivConnection {
    ReadOnly {
        conn: Connection,
        path_manager: Arc<PathManager>,
    },
    Transaction {
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,

        fs_tx: FsTransaction,
        lock_file: LockFile,

        finished: bool,
    },
}

impl ArhivConnection {
    pub fn new(conn: Connection, path_manager: Arc<PathManager>) -> Self {
        ArhivConnection::ReadOnly { conn, path_manager }
    }

    pub fn new_tx(
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,
    ) -> Result<Self> {
        conn.execute_batch("BEGIN DEFERRED")?;

        let lock_file = LockFile::open(&path_manager.lock_file)?;

        Ok(ArhivConnection::Transaction {
            conn,
            schema,
            finished: false,
            path_manager,
            fs_tx: FsTransaction::new(),
            lock_file,
        })
    }

    fn complete_tx(&mut self, commit: bool) -> Result<()> {
        match self {
            ArhivConnection::Transaction {
                finished,
                fs_tx,
                conn,
                ..
            } => {
                ensure!(!*finished, "transaction must not be completed");

                *finished = true;

                if commit {
                    fs_tx.commit()?;
                    conn.execute_batch("COMMIT")?;
                } else {
                    fs_tx.rollback()?;
                    conn.execute_batch("ROLLBACK")?;
                }
            }

            ArhivConnection::ReadOnly { .. } => bail!("not a transaction"),
        };

        Ok(())
    }

    pub fn commit(mut self) -> Result<()> {
        self.complete_tx(true)
    }

    pub fn rollback(&mut self) -> Result<()> {
        self.complete_tx(false)
    }

    fn get_schema(&self) -> Result<Arc<DataSchema>> {
        match self {
            ArhivConnection::Transaction { schema, .. } => Ok(schema.clone()),
            ArhivConnection::ReadOnly { .. } => bail!("not a transaction"),
        }
    }

    pub fn stage_document(&self, document: &mut Document) -> Result<()> {
        log::debug!("Staging document {}", &document.id);

        ensure!(
            !document.is_erased(),
            "erased documents must not be updated"
        );

        let prev_document = self.get_document(&document.id)?;

        let schema = self.get_schema()?;
        let data_description = schema.get_data_description(&document.document_type)?;

        Validator::default().validate(
            &document.data,
            prev_document.as_ref().map(|document| &document.data),
            data_description,
            self,
        )?;

        if let Some(prev_document) = prev_document {
            log::debug!("Updating existing document {}", &document.id);

            document.rev = Revision::STAGING;

            if prev_document.rev == Revision::STAGING {
                ensure!(
                    document.prev_rev == prev_document.prev_rev,
                    "document prev_rev {} is different from the staged document prev_rev {}",
                    document.prev_rev,
                    prev_document.prev_rev
                );
            } else {
                // we're going to modify committed document
                // so we need to save its revision as prev_rev of the new document
                document.prev_rev = prev_document.rev;
            }

            ensure!(
                document.document_type == prev_document.document_type,
                "document type '{}' is different from the type '{}' of existing document",
                document.document_type,
                prev_document.document_type
            );

            ensure!(
                document.created_at == prev_document.created_at,
                "document created_at '{}' is different from the created_at '{}' of existing document",
                document.created_at,
                prev_document.created_at
            );

            ensure!(
                document.updated_at == prev_document.updated_at,
                "document updated_at '{}' is different from the updated_at '{}' of existing document",
                document.updated_at,
                prev_document.updated_at
            );

            document.updated_at = Utc::now();
        } else {
            log::debug!("Creating new document {}", &document.id);

            document.rev = Revision::STAGING;
            document.prev_rev = Revision::STAGING;

            let now = Utc::now();
            document.created_at = now;
            document.updated_at = now;
        }

        self.put_document(document)?;

        log::info!("saved document {}", document);

        Ok(())
    }

    pub fn erase_document(&self, id: &Id) -> Result<()> {
        let mut document = self
            .get_document(id)?
            .ok_or_else(|| anyhow!("can't find document {}", &id))?;

        ensure!(
            !document.is_erased(),
            "erased documents must not be updated"
        );

        document.erase();

        self.put_document(&document)?;

        log::info!("erased document {}", document);

        Ok(())
    }

    pub fn remove_orphaned_blobs(&mut self) -> Result<()> {
        ensure!(
            !self.has_staged_documents()?,
            "there must be no staged changes"
        );

        let used_blob_ids = self.get_used_blob_ids()?;

        let mut removed_blobs = 0;
        for blob_id in self.list_local_blobs()? {
            if !used_blob_ids.contains(&blob_id) {
                self.remove_blob(&blob_id)?;
                removed_blobs += 1;
            }
        }

        log::debug!("Removed {} orphaned blobs", removed_blobs);

        Ok(())
    }

    pub(crate) fn apply_migrations(&self) -> Result<()> {
        let schema_version = self.get_setting(SETTING_SCHEMA_VERSION)?;

        let schema = self.get_schema()?;
        let migrations: Vec<_> = schema
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

        let new_schema_version = schema.get_version();

        let conn = self.get_connection();

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

        conn.create_scalar_function(
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

        let now = Instant::now();

        let rows_count = self.get_connection().execute(
            "UPDATE documents_snapshots
                SET data = apply_migrations(type, data)
                WHERE data <> apply_migrations(type, data)",
            [],
        )?;

        log::info!(
            "Migrated {} rows in {} seconds",
            rows_count,
            now.elapsed().as_secs_f32()
        );

        self.set_setting(SETTING_SCHEMA_VERSION, new_schema_version)?;

        log::info!(
            "Finished schema migration from version {} to {}",
            schema_version,
            new_schema_version
        );

        Ok(())
    }
}

impl Drop for ArhivConnection {
    fn drop(&mut self) {
        match self {
            ArhivConnection::Transaction {
                lock_file,
                finished,
                ..
            } => {
                if lock_file.owns_lock() {
                    if let Err(err) = lock_file.unlock() {
                        log::error!("Failed to unlock arhiv lock file: {}", err);
                    }
                }

                if *finished {
                    return;
                }

                log::warn!("Transaction wasn't committed, rolling back");

                if let Err(err) = self.rollback() {
                    log::error!("Transaction rollback failed: {}", err);
                }
            }

            ArhivConnection::ReadOnly { .. } => {}
        };
    }
}

impl Queries for ArhivConnection {
    fn get_connection(&self) -> &Connection {
        match self {
            ArhivConnection::ReadOnly { conn, .. } | ArhivConnection::Transaction { conn, .. } => {
                conn
            }
        }
    }
}

impl BLOBQueries for ArhivConnection {
    fn get_data_dir(&self) -> &str {
        match self {
            ArhivConnection::ReadOnly { path_manager, .. }
            | ArhivConnection::Transaction { path_manager, .. } => &path_manager.data_dir,
        }
    }
}

impl MutableQueries for ArhivConnection {}

impl MutableBLOBQueries for ArhivConnection {
    fn get_fs_tx(&mut self) -> Result<&mut FsTransaction> {
        match self {
            ArhivConnection::Transaction {
                lock_file,
                ref mut fs_tx,
                ..
            } => {
                if !lock_file.owns_lock() {
                    lock_file
                        .lock()
                        .context("failed to lock on arhiv lock file")?;
                }

                Ok(fs_tx)
            }
            ArhivConnection::ReadOnly { .. } => bail!("not a transaction"),
        }
    }
}
