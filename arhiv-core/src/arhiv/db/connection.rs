use std::{collections::HashSet, fs, sync::Arc, time::Instant};

use anyhow::{anyhow, bail, ensure, Context, Result};
use chrono::Utc;
use fslock::LockFile;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};

use rs_utils::{file_exists, is_same_filesystem, log, FsTransaction};

use crate::{
    arhiv::{db_migrations::get_db_version, status::Status},
    entities::{
        BLOBId, Changeset, ChangesetResponse, Document, Id, Revision, Timestamp, BLOB,
        ERASED_DOCUMENT_TYPE,
    },
    path_manager::PathManager,
    schema::DataSchema,
    Validator,
};

use super::{
    db::{init_functions, open_connection},
    dto::{
        BLOBSCount, DBSetting, DbStatus, DocumentsCount, ListPage, SETTING_ARHIV_ID,
        SETTING_DATA_VERSION, SETTING_IS_PRIME, SETTING_LAST_SYNC_TIME,
    },
    filter::{Filter, OrderBy},
    query_builder::QueryBuilder,
    utils,
};

pub enum ArhivConnection {
    ReadOnly {
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,
    },
    Transaction {
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,

        fs_tx: FsTransaction,
        lock_file: LockFile,

        completed: bool,
    },
}

impl ArhivConnection {
    pub fn new(path_manager: Arc<PathManager>, schema: Arc<DataSchema>) -> Result<Self> {
        let conn = open_connection(&path_manager.db_file, false)?;

        init_functions(&conn, &schema)?;

        Ok(ArhivConnection::ReadOnly {
            conn,
            path_manager,
            schema,
        })
    }

    pub fn new_tx(path_manager: Arc<PathManager>, schema: Arc<DataSchema>) -> Result<Self> {
        let conn = open_connection(&path_manager.db_file, true)?;

        init_functions(&conn, &schema)?;

        conn.execute_batch("BEGIN DEFERRED")?;

        let lock_file = LockFile::open(&path_manager.lock_file)?;

        Ok(ArhivConnection::Transaction {
            conn,
            schema,
            completed: false,
            path_manager,
            fs_tx: FsTransaction::new(),
            lock_file,
        })
    }

    fn complete_tx(&mut self, commit: bool) -> Result<()> {
        match self {
            ArhivConnection::Transaction {
                completed,
                fs_tx,
                conn,
                ..
            } => {
                ensure!(!*completed, "transaction must not be completed");

                *completed = true;

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

    pub(crate) fn get_schema(&self) -> Arc<DataSchema> {
        match self {
            ArhivConnection::Transaction { schema, .. }
            | ArhivConnection::ReadOnly { schema, .. } => schema.clone(),
        }
    }

    pub(crate) fn get_path_manager(&self) -> &PathManager {
        match self {
            ArhivConnection::ReadOnly { path_manager, .. }
            | ArhivConnection::Transaction { path_manager, .. } => path_manager,
        }
    }

    pub(crate) fn get_connection(&self) -> &Connection {
        match self {
            ArhivConnection::ReadOnly { conn, .. } | ArhivConnection::Transaction { conn, .. } => {
                conn
            }
        }
    }

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

    pub fn get_db_status(&self) -> Result<DbStatus> {
        Ok(DbStatus {
            arhiv_id: self.get_setting(&SETTING_ARHIV_ID)?,
            is_prime: self.get_setting(&SETTING_IS_PRIME)?,
            data_version: self.get_setting(&SETTING_DATA_VERSION)?,
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

    fn count_documents(&self) -> Result<DocumentsCount> {
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
                    IFNULL(SUM(CASE WHEN document_type != ?1  AND rev > 0                  THEN 1 ELSE 0 END), 0) AS documents_committed,
                    IFNULL(SUM(CASE WHEN document_type != ?1  AND rev = 0 AND prev_rev > 0 THEN 1 ELSE 0 END), 0) AS documents_updated,
                    IFNULL(SUM(CASE WHEN document_type != ?1  AND rev = 0 AND prev_rev = 0 THEN 1 ELSE 0 END), 0) AS documents_new,

                    IFNULL(SUM(CASE WHEN document_type  = ?1  AND rev > 0                  THEN 1 ELSE 0 END), 0) AS erased_documents_committed,
                    IFNULL(SUM(CASE WHEN document_type  = ?1  AND rev = 0                  THEN 1 ELSE 0 END), 0) AS erased_documents_staged
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

    fn count_conflicts(&self) -> Result<u32> {
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
    pub fn list_documents(&self, filter: &Filter) -> Result<ListPage> {
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
                "calculate_search_score(documents.document_type, documents.subtype, documents.data, {}) AS search_score",
                qb.param(pattern)
            ));
            qb.where_condition("search_score > 0");

            qb.order_by("search_score", false);
        }

        if let Some(ref document_type) = filter.conditions.document_type {
            qb.where_condition(format!(
                "documents.document_type = {}",
                qb.param(document_type)
            ));
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

    pub fn get_documents(&self, ids: &HashSet<&Id>) -> Result<Vec<Document>> {
        ensure!(!ids.is_empty(), "set of ids must not be empty");

        let mut stmt = self.get_connection().prepare_cached(&format!(
            "SELECT * FROM documents WHERE id IN ({}) LIMIT {}",
            vec!["?"; ids.len()].join(", "),
            ids.len()
        ))?;

        let rows = stmt
            .query_and_then(params_from_iter(ids), utils::extract_document)
            .context("failed to get documents")?;

        let documents = rows.collect::<Result<Vec<_>>>()?;

        ensure!(
            documents.len() == ids.len(),
            "expected to get {} documents but got only {}",
            documents.len(),
            ids.len()
        );

        Ok(documents)
    }

    pub(crate) fn get_document_by_rowid(&self, rowid: i64) -> Result<Document> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents_snapshots WHERE rowid = ?1 LIMIT 1")?;

        let mut rows = stmt
            .query_and_then([rowid], utils::extract_document)
            .context(anyhow!(
                "Failed to get document snapshot with rowid {}",
                rowid
            ))?;

        rows.next().expect("document must exist")
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

    fn get_used_blob_ids(&self) -> Result<HashSet<BLOBId>> {
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

    fn count_blobs(&self) -> Result<BLOBSCount> {
        let new_blobs_count: u32 = self
            .get_connection()
            .query_row("SELECT COUNT(*) FROM new_blob_ids", [], |row| row.get(0))
            .context("failed to count new blob ids")?;

        let used_blob_ids = self.get_used_blob_ids()?;
        let local_blob_ids = self.get_local_blob_ids()?;

        let local_used_blob_ids = local_blob_ids
            .intersection(&used_blob_ids)
            .collect::<HashSet<_>>();

        Ok(BLOBSCount {
            blobs_staged: new_blobs_count,
            local_blobs_count: local_blob_ids.len() as u32,
            local_used_blobs_count: local_used_blob_ids.len() as u32,
            total_blobs_count: used_blob_ids.len() as u32,
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
        self.put_or_replace_document(document, false)
    }

    pub(crate) fn put_or_replace_document(
        &self,
        document: &Document,
        force_update: bool,
    ) -> Result<()> {
        {
            let mut stmt = self.get_connection().prepare_cached(&format!(
                "INSERT {} INTO documents_snapshots
                    (id, rev, prev_rev, document_type, subtype, created_at, updated_at, data)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                if force_update || document.is_staged() {
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
                document.subtype,
                document.created_at,
                document.updated_at,
                document.data.to_string(),
            ])
            .context(anyhow!("Failed to put document {}", &document.id))?;
        }

        {
            let mut stmt = self.get_connection().prepare_cached(
                "INSERT OR REPLACE INTO documents_refs (id, rev, refs) VALUES (?, ?, extract_refs(?, ?, ?))"
            )?;

            stmt.execute(params![
                document.id,
                document.rev,
                document.document_type,
                document.subtype,
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
               SELECT id, rev, extract_refs(document_type, subtype, data)
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
        let data_version = self.get_setting(&SETTING_DATA_VERSION)?;
        let documents_count = self.count_documents()?;
        let blobs_count = self.count_blobs()?;
        let conflicts_count = self.count_conflicts()?;
        let last_update_time = self.get_last_update_time()?;

        Ok(Status {
            db_status,
            db_version,
            data_version,
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

        Validator::new(self).validate(
            document,
            prev_document.as_ref().map(|document| &document.data),
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

    pub(crate) fn generate_changeset(&self) -> Result<Changeset> {
        let db_status = self.get_db_status()?;

        let documents = self.list_documents(&Filter::all_staged_documents())?.items;

        let changeset = Changeset {
            data_version: self.get_setting(&SETTING_DATA_VERSION)?,
            arhiv_id: db_status.arhiv_id,
            base_rev: db_status.db_rev,
            documents,
        };

        Ok(changeset)
    }

    pub(crate) fn generate_changeset_response(
        &self,
        base_rev: Revision,
        conflicts: Vec<Document>,
    ) -> Result<ChangesetResponse> {
        let next_rev = base_rev.inc();

        let new_snapshots = self.get_new_snapshots_since(next_rev)?;

        let arhiv_id = self.get_setting(&SETTING_ARHIV_ID)?;
        let latest_rev = self.get_db_rev()?;

        Ok(ChangesetResponse {
            arhiv_id,
            base_rev,
            latest_rev,
            new_snapshots,
            conflicts,
        })
    }

    pub(crate) fn apply_changeset_response(&mut self, response: ChangesetResponse) -> Result<()> {
        let db_status = self.get_db_status()?;

        ensure!(
            response.arhiv_id == db_status.arhiv_id,
            "changeset response arhiv_id {} isn't equal to current arhiv_id {}",
            response.arhiv_id,
            db_status.arhiv_id,
        );
        ensure!(
            response.base_rev == db_status.db_rev,
            "base_rev {} isn't equal to current rev {}",
            response.base_rev,
            db_status.db_rev,
        );

        for document in response.new_snapshots {
            self.put_document(&document)?;

            // erase history of erased documents
            if document.is_erased() {
                self.erase_document_history(&document.id)?;
            }
        }

        if !response.conflicts.is_empty() {
            log::warn!(
                "Got {} conflict(s) in changeset response",
                response.conflicts.len()
            );
        }

        // save conflicts in documents table
        for document in response.conflicts {
            log::warn!("Conflict in {}", &document);
            self.put_document(&document)?;
        }

        log::debug!("successfully applied a changeset response");

        Ok(())
    }

    pub(crate) fn get_blob(&self, blob_id: &BLOBId) -> BLOB {
        BLOB::new(blob_id.clone(), &self.get_path_manager().data_dir)
    }

    pub(crate) fn get_local_blob_ids(&self) -> Result<HashSet<BLOBId>> {
        let items = fs::read_dir(&self.get_path_manager().data_dir)?
            .map(|item| {
                let entry = item.context("Failed to read data entry")?;

                let entry_path = entry.path();

                ensure!(
                    entry_path.is_file(),
                    "{} isn't a file",
                    entry_path.to_string_lossy()
                );

                entry_path
                    .file_name()
                    .ok_or_else(|| anyhow!("Failed to read file name"))
                    .map(|value| value.to_string_lossy().to_string())
                    .and_then(|value| BLOBId::from_file_name(&value))
            })
            .collect::<Result<HashSet<_>>>()?;

        Ok(items)
    }

    pub(crate) fn get_missing_blob_ids(&self) -> Result<HashSet<BLOBId>> {
        let used_blob_ids = self.get_used_blob_ids()?;
        let local_blob_ids = self.get_local_blob_ids()?;

        let missing_blob_ids = used_blob_ids
            .into_iter()
            .filter(|blob_id| !local_blob_ids.contains(blob_id))
            .collect();

        Ok(missing_blob_ids)
    }

    pub fn add_blob(&mut self, file_path: &str, move_file: bool) -> Result<BLOBId> {
        ensure!(
            file_exists(file_path)?,
            "BLOB source must exist and must be a file"
        );

        let blob_id = BLOBId::from_file(file_path)?;

        let blob = self.get_blob(&blob_id);

        if blob.exists()? {
            log::debug!("blob {} already exists", blob_id);

            return Ok(blob_id);
        }

        let data_dir = self.get_path_manager().data_dir.clone();
        let fs_tx = self.get_fs_tx()?;

        if move_file {
            fs_tx.move_file(file_path, blob.file_path)?;
            log::debug!("Moved new blob {} from {}", blob_id, file_path);
        } else if is_same_filesystem(file_path, &data_dir)? {
            fs_tx.hard_link_file(file_path, blob.file_path)?;
            log::debug!("Hard linked new blob {} from {}", blob_id, file_path);
        } else {
            fs_tx.copy_file(file_path, blob.file_path)?;
            log::debug!("Copied new blob {} from {}", blob_id, file_path);
        }

        log::info!("Created blob {} from {}", &blob_id, file_path);

        Ok(blob_id)
    }

    fn remove_blob(&mut self, blob_id: &BLOBId) -> Result<()> {
        let blob = self.get_blob(blob_id);

        self.get_fs_tx()?.remove_file(&blob.file_path)?;

        log::debug!("Removed blob {} from {}", blob_id, blob.file_path);

        Ok(())
    }

    pub(crate) fn remove_orphaned_blobs(&mut self) -> Result<()> {
        ensure!(
            !self.has_staged_documents()?,
            "there must be no staged changes"
        );

        let used_blob_ids = self.get_used_blob_ids()?;

        let mut removed_blobs = 0;
        for blob_id in self.get_local_blob_ids()? {
            if !used_blob_ids.contains(&blob_id) {
                self.remove_blob(&blob_id)?;
                removed_blobs += 1;
            }
        }

        log::debug!("Removed {} orphaned blobs", removed_blobs);

        Ok(())
    }

    // FIXME pub fn get_blob_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_blob_stream(&self, hash: &hash, stream: FileStream) -> Result<()>
}

impl Drop for ArhivConnection {
    fn drop(&mut self) {
        match self {
            ArhivConnection::Transaction {
                lock_file,
                completed,
                ..
            } => {
                if lock_file.owns_lock() {
                    if let Err(err) = lock_file.unlock() {
                        log::error!("Failed to unlock arhiv lock file: {}", err);
                    }
                }

                if *completed {
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
