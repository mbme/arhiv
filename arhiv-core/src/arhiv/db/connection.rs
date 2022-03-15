use std::sync::Arc;

use anyhow::{anyhow, ensure, Result};
use chrono::Utc;
use rusqlite::Connection;

use rs_utils::{log, FsTransaction};

use crate::{
    entities::{Document, Id, Revision},
    schema::DataSchema,
    Validator,
};

use super::{blob_queries::*, queries::*};

pub struct ArhivConnection {
    conn: Connection,
    data_dir: String,
}

impl<'a> ArhivConnection {
    pub fn new(conn: Connection, data_dir: String) -> Self {
        ArhivConnection { conn, data_dir }
    }
}

pub struct ArhivTransaction {
    pub(crate) conn: Connection,
    schema: Arc<DataSchema>,
    finished: bool,

    data_dir: String,
    fs_tx: FsTransaction,
}

impl ArhivTransaction {
    pub fn new(conn: Connection, data_dir: String, schema: Arc<DataSchema>) -> Result<Self> {
        conn.execute_batch("BEGIN DEFERRED")?;

        Ok(ArhivTransaction {
            conn,
            schema,
            finished: false,
            data_dir,
            fs_tx: FsTransaction::new(),
        })
    }

    pub fn commit(mut self) -> Result<()> {
        ensure!(
            !self.finished,
            "must not try to commit finished transaction"
        );

        self.finished = true;

        self.fs_tx.commit()?;
        self.conn.execute_batch("COMMIT")?;

        Ok(())
    }

    pub fn rollback(&mut self) -> Result<()> {
        ensure!(
            !self.finished,
            "must not try to rollback finished transaction"
        );

        self.finished = true;

        self.conn.execute_batch("ROLLBACK")?;

        Ok(())
    }

    pub fn stage_document(&self, document: &mut Document) -> Result<()> {
        log::debug!("Staging document {}", &document.id);

        ensure!(
            !document.is_erased(),
            "erased documents must not be updated"
        );

        let prev_document = self.get_document(&document.id)?;

        let schema = self.schema.clone();
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
}

impl Drop for ArhivTransaction {
    fn drop(&mut self) {
        if self.finished {
            return;
        }

        log::warn!("Transaction wasn't committed, rolling back");

        if let Err(err) = self.rollback() {
            log::error!("Transaction rollback failed: {}", err);
        }
    }
}

impl Queries for ArhivConnection {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl BLOBQueries for ArhivConnection {
    fn get_data_dir(&self) -> &str {
        &self.data_dir
    }
}

impl Queries for ArhivTransaction {
    fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

impl BLOBQueries for ArhivTransaction {
    fn get_data_dir(&self) -> &str {
        &self.data_dir
    }
}

impl MutableQueries for ArhivTransaction {}

impl MutableBLOBQueries for ArhivTransaction {
    fn get_fs_tx(&mut self) -> &mut FsTransaction {
        &mut self.fs_tx
    }
}
