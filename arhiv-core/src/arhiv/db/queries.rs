use std::{collections::HashSet, sync::Arc, time::Instant};

use anyhow::{anyhow, ensure, Context, Result};
use chrono::Utc;
use rusqlite::{
    functions::{Context as FunctionContext, FunctionFlags},
    params, params_from_iter, Error as RusqliteError, OptionalExtension,
};
use serde::{de::DeserializeOwned, Serialize};

use rs_utils::log;

use crate::{
    arhiv::{migrations::get_db_version, status::Status},
    entities::{BLOBId, Document, DocumentData, Id, Revision, Timestamp, ERASED_DOCUMENT_TYPE},
    Validator,
};

use super::{
    dto::{
        BLOBSCount, DBSetting, DbStatus, DocumentsCount, ListPage, SETTING_ARHIV_ID,
        SETTING_IS_PRIME, SETTING_LAST_SYNC_TIME, SETTING_SCHEMA_VERSION,
    },
    filter::{Filter, OrderBy},
    query_builder::QueryBuilder,
    utils, ArhivConnection,
};

impl ArhivConnection {
    pub(crate) fn get_setting<T: Serialize + DeserializeOwned>(
        &self,
        setting: &DBSetting<T>,
    ) -> Result<T> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT value FROM settings WHERE key = ?1")?;

        let value: String = stmt
            .query_row([setting.0], |row| row.get(0))
            .context(anyhow!("failed to read setting {}", setting.0))?;

        serde_json::from_str(&value).context(anyhow!("failed to parse setting {}", setting.0))
    }

    pub(crate) fn get_db_status(&self) -> Result<DbStatus> {
        Ok(DbStatus {
            arhiv_id: self.get_setting(&SETTING_ARHIV_ID)?,
            is_prime: self.get_setting(&SETTING_IS_PRIME)?,
            schema_version: self.get_setting(&SETTING_SCHEMA_VERSION)?,
            db_rev: self.get_db_rev()?,
            last_sync_time: self.get_setting(&SETTING_LAST_SYNC_TIME)?,
        })
    }

    pub(crate) fn get_db_rev(&self) -> Result<Revision> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT IFNULL(MAX(rev), 0) FROM documents_snapshots")?;

        stmt.query_row([], |row| row.get(0))
            .context("failed to query for db rev")
    }

    pub(crate) fn count_documents(&self) -> Result<DocumentsCount> {
        // count documents
        // count erased documents
        let conn = self.get_connection();

        let (
            documents_committed,
            documents_updated,
            documents_new,
            erased_documents_committed,
            erased_documents_staged,
        ): (u32, u32, u32, u32, u32) = conn
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN type != ?1  AND rev > 0                  THEN 1 ELSE 0 END), 0) AS documents_committed,
                    IFNULL(SUM(CASE WHEN type != ?1  AND rev = 0 AND prev_rev > 0 THEN 1 ELSE 0 END), 0) AS documents_updated,
                    IFNULL(SUM(CASE WHEN type != ?1  AND rev = 0 AND prev_rev = 0 THEN 1 ELSE 0 END), 0) AS documents_new,

                    IFNULL(SUM(CASE WHEN type  = ?1  AND rev > 0                  THEN 1 ELSE 0 END), 0) AS erased_documents_committed,
                    IFNULL(SUM(CASE WHEN type  = ?1  AND rev = 0                  THEN 1 ELSE 0 END), 0) AS erased_documents_staged
                FROM documents",
                [ERASED_DOCUMENT_TYPE],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .context("Failed to count documents")?;

        let snapshots: u32 = conn
            .query_row("SELECT COUNT(*) FROM documents_snapshots", [], |row| {
                row.get(0)
            })
            .context("Failed to count documents_snapshots")?;

        Ok(DocumentsCount {
            documents_committed,
            documents_updated,
            documents_new,

            erased_documents_committed,
            erased_documents_staged,

            snapshots,
        })
    }

    pub(crate) fn count_conflicts(&self) -> Result<u32> {
        self.get_connection()
            .query_row("SELECT COUNT(*) FROM conflicts", [], |row| row.get(0))
            .context("failed to count conflicts")
    }

    pub(crate) fn has_staged_documents(&self) -> Result<bool> {
        self.get_connection()
            .query_row(
                "SELECT true FROM documents_snapshots WHERE rev = 0 LIMIT 1",
                [],
                |_row| Ok(true),
            )
            .optional()
            .context("Failed to check for staged documents")
            .map(|value| value.unwrap_or(false))
    }

    pub(crate) fn get_last_update_time(&self) -> Result<Timestamp> {
        let result: Option<Timestamp> = self
            .get_connection() // FIXME check if this ordering actually works
            .query_row(
                "SELECT updated_at FROM documents_snapshots ORDER BY updated_at DESC LIMIT 1",
                [],
                |row| Ok(row.get_unwrap(0)),
            )
            .optional()
            .context("Failed to get last update time")?;

        Ok(result.unwrap_or(chrono::MIN_DATETIME))
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn list_documents(&self, filter: &Filter) -> Result<ListPage<Document>> {
        let mut qb = QueryBuilder::new();

        qb.select("*", "documents");

        if let Some(true) = filter.conditions.only_staged {
            qb.where_condition("documents.rev = 0");
        }

        if let Some((ref field, ref pattern)) = filter.conditions.field {
            qb.where_condition(format!(
                "json_contains(documents.data, {}, {})",
                qb.param(field),
                qb.param(pattern)
            ));
        }

        if let Some(ref pattern) = filter.conditions.search {
            qb.and_select(format!(
                "calculate_search_score(documents.type, documents.data, {}) AS search_score",
                qb.param(pattern)
            ));
            qb.where_condition("search_score > 0");

            qb.order_by("search_score", false);
        }

        if let Some(ref document_type) = filter.conditions.document_type {
            qb.where_condition(format!("documents.type = {}", qb.param(document_type)));
        }

        if let Some(ref id) = filter.conditions.document_ref {
            qb.and_from("json_each(refs, '$.documents') AS document_refs");
            qb.where_condition(format!("document_refs.value = {}", qb.param(id.clone())));
        }

        if let Some(ref collection_id) = filter.conditions.collection_ref {
            qb.and_from("json_each(refs, '$.collections') AS collection_refs");
            qb.where_condition(format!(
                "collection_refs.value = {}",
                qb.param(collection_id.clone())
            ));
        }

        for order in &filter.order {
            match order {
                OrderBy::UpdatedAt { asc } => {
                    qb.order_by("documents.updated_at", *asc);
                }
                OrderBy::Field { ref selector, asc } => {
                    qb.order_by(
                        format!("json_extract(documents.data, {})", qb.param(selector)),
                        *asc,
                    );
                }
                OrderBy::EnumField {
                    selector,
                    asc,
                    enum_order,
                } => {
                    let cases = enum_order
                        .iter()
                        .enumerate()
                        .map(|(j, item)| format!("WHEN {} THEN {}", qb.param(item), j))
                        .collect::<Vec<String>>()
                        .join(" ");

                    qb.order_by(
                        format!(
                            "CASE json_extract(documents.data, {}) {} ELSE {} END",
                            qb.param(selector),
                            cases,
                            enum_order.len(),
                        ),
                        *asc,
                    );
                }
            }
        }

        let mut page_size: i32 = -1;
        match (filter.page_size, filter.page_offset) {
            (None, None) => {}
            (page_size_opt, page_offset_opt) => {
                page_size = page_size_opt.map_or(-1, |val| val as i32);

                // fetch (page_size + 1) items so that we know that there are more items than page_size
                if page_size > -1 {
                    page_size += 1;
                }

                qb.limit(page_size);

                let page_offset = page_offset_opt.unwrap_or(0);

                qb.offset(page_offset as u32);
            }
        }

        let (query, params) = qb.build();
        log::debug!("list_documents: {}", &query);

        let mut stmt = self.get_connection().prepare(&query)?;

        let mut rows = stmt.query(params_from_iter(params))?;

        let mut items = Vec::new();
        let mut has_more = false;
        while let Some(row) = rows.next()? {
            if page_size > -1 && items.len() as i32 == page_size - 1 {
                has_more = true;
                break; // due to break we ignore last item
            }

            items.push(utils::extract_document(row)?);
        }

        Ok(ListPage { items, has_more })
    }

    pub(crate) fn get_new_snapshots_since(&self, min_rev: Revision) -> Result<Vec<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents_snapshots WHERE rev >= ?1")?;

        let rows = stmt
            .query_and_then([min_rev], utils::extract_document)
            .context(anyhow!("Failed to get new snapshots since {}", min_rev))?;

        let mut documents = Vec::new();
        for row in rows {
            documents.push(row?);
        }

        Ok(documents)
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents WHERE id = ?1 LIMIT 1")?;

        let mut rows = stmt
            .query_and_then([id], utils::extract_document)
            .context(anyhow!("Failed to get document {}", &id))?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_new_blob_ids(&self) -> Result<HashSet<BLOBId>> {
        let mut stmt = self
            .get_connection()
            .prepare("SELECT blob_id FROM new_blob_ids")?;

        let rows = stmt.query_and_then([], utils::extract_blob_id)?;

        let mut result = HashSet::new();
        for entry in rows {
            result.insert(entry?);
        }

        Ok(result)
    }

    pub(crate) fn get_used_blob_ids(&self) -> Result<HashSet<BLOBId>> {
        let mut stmt = self
            .get_connection()
            .prepare("SELECT blob_id FROM used_blob_ids")?;

        let rows = stmt.query_and_then([], utils::extract_blob_id)?;

        let mut result = HashSet::new();
        for entry in rows {
            result.insert(entry?);
        }

        Ok(result)
    }

    pub(crate) fn count_blobs(&self) -> Result<BLOBSCount> {
        let committed_blobs_count: u32 = self
            .get_connection()
            .query_row("SELECT COUNT(*) FROM committed_blob_ids", [], |row| {
                row.get(0)
            })
            .context("failed to count used blob ids")?;

        let new_blobs_count: u32 = self
            .get_connection()
            .query_row("SELECT COUNT(*) FROM new_blob_ids", [], |row| row.get(0))
            .context("failed to count new blob ids")?;

        Ok(BLOBSCount {
            blobs_committed: committed_blobs_count,
            blobs_staged: new_blobs_count,
        })
    }

    pub(crate) fn is_known_blob_id(&self, blob_id: &BLOBId) -> Result<bool> {
        self.get_connection()
            .query_row(
                "SELECT true FROM used_blob_ids WHERE blob_id = ?1 LIMIT 1",
                params![blob_id],
                |_row| Ok(true),
            )
            .optional()
            .context("Failed to check if BLOB id is known")
            .map(|value| value.unwrap_or(false))
    }

    pub(crate) fn has_snapshot(&self, id: &Id, rev: Revision) -> Result<bool> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT true FROM documents_snapshots WHERE id = ?1 AND rev = ?2")?;

        stmt.query_row(params![id, rev], |_row| Ok(true))
            .optional()
            .context(anyhow!("Failed to check for snapshot {}", &id))
            .map(|value| value.unwrap_or(false))
    }

    pub(crate) fn get_last_snapshot(&self, id: &Id) -> Result<Option<Document>> {
        let mut stmt = self.get_connection().prepare_cached(
            "SELECT * FROM documents_snapshots WHERE id = ?1 ORDER BY rev DESC LIMIT 1",
        )?;

        let mut rows = stmt
            .query_and_then([id], utils::extract_document)
            .context(anyhow!("Failed to get last snapshot of document {}", &id))?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn set_setting<T: Serialize + DeserializeOwned>(
        &self,
        setting: &DBSetting<T>,
        value: &T,
    ) -> Result<()> {
        let value = serde_json::to_string(&value)?;

        self.get_connection()
            .execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                [setting.0, &value],
            )
            .context(anyhow!("failed to save setting {}", setting.0))?;

        Ok(())
    }

    pub(crate) fn put_document(&self, document: &Document) -> Result<()> {
        {
            let mut stmt = self.get_connection().prepare_cached(&format!(
                "INSERT {} INTO documents_snapshots
                    (id, rev, prev_rev, type, created_at, updated_at, data)
                    VALUES (?, ?, ?, ?, ?, ?, ?)",
                if document.is_staged() {
                    "OR REPLACE"
                } else {
                    ""
                },
            ))?;

            stmt.execute(params![
                document.id,
                document.rev,
                document.prev_rev,
                document.document_type,
                document.created_at,
                document.updated_at,
                document.data.to_string(),
            ])
            .context(anyhow!("Failed to put document {}", &document.id))?;
        }

        {
            let mut stmt = self.get_connection().prepare_cached(
                "INSERT OR REPLACE INTO documents_refs (id, rev, refs) VALUES (?, ?, extract_refs(?, ?))"
            )?;

            stmt.execute(params![
                document.id,
                document.rev,
                document.document_type,
                document.data.to_string(),
            ])
            .context(anyhow!("Failed to put document refs {}", &document.id))?;
        }

        Ok(())
    }

    // delete all document versions except the latest one
    pub(crate) fn erase_document_history(&self, id: &Id) -> Result<()> {
        let rows_count = self.get_connection().execute(
            "DELETE FROM documents_snapshots
             WHERE id = ?1 AND rev <> (SELECT MAX(rev) FROM documents_snapshots WHERE id = ?1)",
            [id],
        )?;

        log::debug!("erased {} rows of history for document {}", rows_count, id);

        Ok(())
    }

    pub(crate) fn delete_local_staged_changes(&self) -> Result<()> {
        self.get_connection()
            .execute("DELETE FROM documents_snapshots WHERE rev = 0", [])?;

        Ok(())
    }

    pub(crate) fn compute_data(&self) -> Result<()> {
        let now = Instant::now();

        let rows_count = self.get_connection().execute(
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

    pub fn get_status(&self) -> Result<Status> {
        let root_dir = self.get_path_manager().root_dir.clone();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let db_status = self.get_db_status()?;
        let db_version = get_db_version(self.get_connection())?;
        let schema_version = self.get_setting(&SETTING_SCHEMA_VERSION)?;
        let documents_count = self.count_documents()?;
        let blobs_count = self.count_blobs()?;
        let conflicts_count = self.count_conflicts()?;
        let last_update_time = self.get_last_update_time()?;

        Ok(Status {
            db_status,
            db_version,
            schema_version,
            documents_count,
            blobs_count,
            conflicts_count,
            last_update_time,
            debug_mode,
            root_dir,
        })
    }

    pub fn stage_document(&self, document: &mut Document) -> Result<()> {
        log::debug!("Staging document {}", &document.id);

        ensure!(
            !document.is_erased(),
            "erased documents must not be updated"
        );

        let prev_document = self.get_document(&document.id)?;

        let schema = self.get_schema();
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

    pub(crate) fn apply_migrations(&self) -> Result<()> {
        let schema_version = self.get_setting(&SETTING_SCHEMA_VERSION)?;

        let schema = self.get_schema();
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

        self.set_setting(&SETTING_SCHEMA_VERSION, &new_schema_version)?;

        log::info!(
            "Finished schema migration from version {} to {}",
            schema_version,
            new_schema_version
        );

        Ok(())
    }
}
