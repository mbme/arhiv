use std::{borrow::Cow, collections::HashSet, sync::Arc, time::Instant};

use anyhow::{anyhow, bail, ensure, Context, Result};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::broadcast::Sender;

use rs_utils::{
    file_exists, is_same_filesystem, log, now, FsTransaction, LockFile, Timestamp, MIN_TIMESTAMP,
};

use crate::{
    baza::BazaEvent,
    entities::{
        BLOBId, Document, DocumentLockKey, Id, InstanceId, Refs, Revision, BLOB,
        ERASED_DOCUMENT_TYPE,
    },
    path_manager::PathManager,
    schema::DataSchema,
    validator::{ValidationError, Validator},
    DocumentExpert,
};

use super::{
    dto::{BLOBSCount, DocumentsCount, ListPage},
    filter::{Filter, OrderBy},
    migrations::get_db_version,
    query_builder::QueryBuilder,
    sqlite_connection::{init_functions, open_connection},
    utils::{self, get_local_blob_ids},
};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum StagingError {
    Validation(#[from] ValidationError),
    Other(#[from] anyhow::Error),
}

pub enum BazaConnection {
    ReadOnly {
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,
    },
    Transaction {
        conn: Connection,
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,

        event_sender: Sender<BazaEvent>,
        events: Vec<BazaEvent>,

        fs_tx: FsTransaction,
        lock_file: LockFile,

        completed: bool,
    },
}

impl BazaConnection {
    pub fn new(path_manager: Arc<PathManager>, schema: Arc<DataSchema>) -> Result<Self> {
        let conn = open_connection(&path_manager.db_file, false)?;

        init_functions(&conn, &schema)?;

        Ok(BazaConnection::ReadOnly {
            conn,
            path_manager,
            schema,
        })
    }

    pub fn new_tx(
        path_manager: Arc<PathManager>,
        schema: Arc<DataSchema>,
        event_sender: Sender<BazaEvent>,
    ) -> Result<Self> {
        let conn = open_connection(&path_manager.db_file, true)?;

        init_functions(&conn, &schema)?;

        conn.execute_batch("BEGIN DEFERRED")?;

        let lock_file = LockFile::new(&path_manager.lock_file)?;

        Ok(BazaConnection::Transaction {
            conn,
            schema,
            completed: false,
            path_manager,
            fs_tx: FsTransaction::new(),
            lock_file,
            event_sender,
            events: Vec::new(),
        })
    }

    fn complete_tx(&mut self, commit: bool) -> Result<()> {
        match self {
            BazaConnection::Transaction {
                completed,
                fs_tx,
                conn,
                event_sender,
                events,
                ..
            } => {
                ensure!(!*completed, "transaction must not be completed");

                *completed = true;

                if commit {
                    fs_tx.commit()?;
                    conn.execute_batch("COMMIT")?;

                    for event in events {
                        if let Err(err) = event_sender.send(event.clone()) {
                            log::warn!("failed to send event: {err}");
                        }
                    }
                } else {
                    fs_tx.rollback()?;
                    conn.execute_batch("ROLLBACK")?;
                }
            }

            BazaConnection::ReadOnly { .. } => bail!("not a transaction"),
        };

        Ok(())
    }

    pub fn commit(mut self) -> Result<()> {
        self.complete_tx(true)
    }

    pub fn rollback(&mut self) -> Result<()> {
        self.complete_tx(false)
    }

    pub fn get_schema(&self) -> Arc<DataSchema> {
        match self {
            BazaConnection::Transaction { schema, .. }
            | BazaConnection::ReadOnly { schema, .. } => schema.clone(),
        }
    }

    pub fn get_path_manager(&self) -> &PathManager {
        match self {
            BazaConnection::ReadOnly { path_manager, .. }
            | BazaConnection::Transaction { path_manager, .. } => path_manager,
        }
    }

    // FIXME make private
    pub fn get_connection(&self) -> &Connection {
        match self {
            BazaConnection::ReadOnly { conn, .. } | BazaConnection::Transaction { conn, .. } => {
                conn
            }
        }
    }

    fn get_fs_tx(&mut self) -> Result<&mut FsTransaction> {
        match self {
            BazaConnection::Transaction { ref mut fs_tx, .. } => Ok(fs_tx),
            BazaConnection::ReadOnly { .. } => bail!("not a transaction"),
        }
    }

    pub(crate) fn register_event(&mut self, event: BazaEvent) -> Result<()> {
        let events = match self {
            BazaConnection::ReadOnly { .. } => {
                bail!("readonly connection doesn't have event sender")
            }
            BazaConnection::Transaction { events, .. } => events,
        };

        events.push(event);

        Ok(())
    }

    pub fn get_db_version(&self) -> Result<u8> {
        get_db_version(self.get_connection())
    }

    pub fn get_db_rev(&self) -> Result<Revision> {
        let mut stmt = self.get_connection().prepare_cached(
            "SELECT rev FROM documents_snapshots WHERE rev != ?1 ORDER BY rev COLLATE REV_CMP DESC LIMIT 1",
        )?;

        let value: Option<Value> = stmt
            .query_row([Revision::STAGED_STRING], |row| row.get(0))
            .optional()
            .context("failed to query for db rev")?;

        if let Some(value) = value {
            Revision::from_value(value)
        } else {
            Ok(Revision::initial())
        }
    }

    pub fn count_documents(&self) -> Result<DocumentsCount> {
        // count documents
        // count erased documents
        let conn = self.get_connection();

        let (
            documents_committed,
            documents_updated,
            documents_new,
            erased_documents_committed,
            erased_documents_staged,
            snapshots,
        ): (u32, u32, u32, u32, u32, u32) = conn
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN document_type != ?1  AND rev != ?2               THEN 1 ELSE 0 END), 0) AS documents_committed,
                    IFNULL(SUM(CASE WHEN document_type != ?1  AND rev =  ?2 AND count > 1 THEN 1 ELSE 0 END), 0) AS documents_updated,
                    IFNULL(SUM(CASE WHEN document_type != ?1  AND rev =  ?2 AND count = 1 THEN 1 ELSE 0 END), 0) AS documents_new,

                    IFNULL(SUM(CASE WHEN document_type  = ?1  AND rev != ?2               THEN 1 ELSE 0 END), 0) AS erased_documents_committed,
                    IFNULL(SUM(CASE WHEN document_type  = ?1  AND rev =  ?2               THEN 1 ELSE 0 END), 0) AS erased_documents_staged,
                    IFNULL(SUM(count), 0)                                                                        AS snapshots
                FROM (SELECT *, COUNT(*) AS count FROM documents_snapshots GROUP BY id)",
                [ERASED_DOCUMENT_TYPE, Revision::STAGED_STRING],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .context("Failed to count documents")?;

        Ok(DocumentsCount {
            documents_committed,
            documents_updated,
            documents_new,

            erased_documents_committed,
            erased_documents_staged,

            snapshots,
        })
    }

    pub fn has_staged_documents(&self) -> Result<bool> {
        self.get_connection()
            .query_row(
                "SELECT true FROM documents_snapshots WHERE rev = ?1 LIMIT 1",
                [Revision::STAGED_STRING],
                |_row| Ok(true),
            )
            .optional()
            .context("Failed to check for staged documents")
            .map(|value| value.unwrap_or(false))
    }

    fn get_max_rev_version(&self, id: &InstanceId) -> Result<u32> {
        self.get_connection()
            .query_row(
                &format!(
                    "SELECT IFNULL(MAX(json_extract(rev, '$.{}')), 0) FROM documents_snapshots",
                    id
                ),
                [],
                |row| row.get::<usize, u32>(0),
            )
            .context("failed to get max rev version")
    }

    pub fn get_last_update_time(&self) -> Result<Timestamp> {
        let result: Option<Timestamp> = self
            .get_connection() // FIXME check if this ordering actually works
            .query_row(
                "SELECT updated_at FROM documents_snapshots ORDER BY updated_at DESC LIMIT 1",
                [],
                |row| Ok(row.get_unwrap(0)),
            )
            .optional()
            .context("Failed to get last update time")?;

        Ok(result.unwrap_or(MIN_TIMESTAMP))
    }

    #[allow(clippy::too_many_lines)]
    pub fn list_documents(&self, filter: &Filter) -> Result<ListPage> {
        let mut qb = QueryBuilder::new();

        qb.select("*", "documents");

        if let Some(true) = filter.conditions.only_staged {
            qb.where_condition(format!("documents.rev = '{}'", Revision::STAGED_STRING));
        }

        if let Some(true) = filter.conditions.skip_erased {
            qb.where_condition("documents.document_type != ''");
        }

        if let Some((ref field, ref pattern)) = filter.conditions.field {
            qb.where_condition(format!(
                "json_contains(documents.data, {}, {})",
                qb.param(field),
                qb.param(pattern)
            ));
        }

        // order of conditions matters!
        if !filter.conditions.document_types.is_empty() {
            let list = filter
                .conditions
                .document_types
                .iter()
                .map(|document_type| format!("'{document_type}'"))
                .collect::<Vec<_>>()
                .join(", ");

            qb.where_condition(format!("documents.document_type IN ({list})"));
        }

        if let Some(ref pattern) = filter.conditions.search {
            qb.and_select(format!(
                "calculate_search_score(documents.document_type, documents.data, {}) AS search_score",
                qb.param(pattern)
            ));
            qb.where_condition("search_score > 0");

            qb.order_by("search_score", false);
        }

        if let Some(ref id) = filter.conditions.document_ref {
            qb.and_from("json_each(refs, '$.documents') AS document_refs");
            qb.where_condition(format!("document_refs.value = {}", qb.param(id.clone())));
        }

        if let Some(ref collection_id) = filter.conditions.collection_ref {
            qb.and_from("json_each(refs, '$.collection') AS collection_refs");
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
        log::trace!("list_documents: {}", &query);

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

    pub fn must_get_document(&self, id: &Id) -> Result<Document> {
        self.get_document(id)?
            .ok_or_else(|| anyhow!("Can't find document with id '{}'", id))
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

    pub fn get_coflicting_documents(&self) -> Result<Vec<Id>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT id FROM documents_with_conflicts")?;

        let rows = stmt
            .query_and_then([], |row| row.get("id").context("failed to get id"))
            .context("failed to get documents")?;

        rows.collect::<Result<Vec<_>>>()
    }

    fn get_document_by_rowid(&self, rowid: &i64) -> Result<Document> {
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

    pub fn get_new_blob_ids(&self) -> Result<HashSet<BLOBId>> {
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

    pub fn count_blobs(&self) -> Result<BLOBSCount> {
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

    pub(crate) fn has_snapshot(&self, id: &Id, rev: &Revision) -> Result<bool> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT true FROM documents_snapshots WHERE id = ?1 AND rev = ?2")?;

        stmt.query_row(params![id, rev.serialize()], |_row| Ok(true))
            .optional()
            .context(anyhow!("Failed to check for snapshot {}", &id))
            .map(|value| value.unwrap_or(false))
    }

    pub(crate) fn put_document(
        &self,
        document: &Document,
        lock_key: Option<DocumentLockKey>,
    ) -> Result<()> {
        let lock = self.get_document_lock(&document.id)?;

        match (lock, lock_key) {
            (Some(lock), Some(lock_key)) => {
                if lock.is_valid_key(&lock_key) {
                    log::debug!("Document is locked, a valid lock key has been provided");
                } else {
                    log::error!("Document is locked, but an invalid lock key has been provided");
                    bail!("Invalid lock key");
                }
            }
            (Some(_), None) => {
                log::error!("Document is locked, but no lock key has been provided");
                bail!("Missing lock key");
            }
            (None, Some(_)) => {
                log::error!("Document isn't locked, but lock key has been provided");
                bail!("Unexpected lock key");
            }
            (None, None) => {
                log::trace!("Document isn't locked, no lock key provided");
            }
        };

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
                    (id, rev, document_type, updated_at, data)
                    VALUES (?, ?, ?, ?, ?)",
                if force_update || document.is_staged() {
                    "OR REPLACE"
                } else {
                    ""
                },
            ))?;

            stmt.execute(params![
                document.id,
                Revision::to_string(&document.rev),
                document.document_type,
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
                Revision::to_string(&document.rev),
                document.document_type,
                document.data.to_string(),
            ])
            .context(anyhow!("Failed to put document refs {}", &document.id))?;
        }

        Ok(())
    }

    pub(crate) fn erase_document_history(&self, id: &Id, rev: &Revision) -> Result<()> {
        let rows_count = self.get_connection().execute(
            "DELETE FROM documents_snapshots
             WHERE id = ?1 AND rev < ?2 COLLATE REV_CMP",
            params![id, rev.serialize()],
        )?;

        log::debug!(
            "erased {} rows of history for document {} up to the rev {}",
            rows_count,
            id,
            rev.serialize()
        );

        Ok(())
    }

    fn delete_local_staged_changes(&self) -> Result<()> {
        self.get_connection().execute(
            "DELETE FROM documents_snapshots WHERE rev = ?1",
            [Revision::STAGED_STRING],
        )?;

        Ok(())
    }

    pub(crate) fn compute_data(&self) -> Result<()> {
        let computed_data_version = self.get_computed_data_version()?;

        ensure!(
            computed_data_version <= Refs::VERSION,
            "computed_data_version is greater than Refs version"
        );
        if computed_data_version < Refs::VERSION {
            self.get_connection()
                .execute("DELETE FROM documents_refs", [])?;
            self.set_computed_data_version(Refs::VERSION)?;
        }

        let now = Instant::now();

        let rows_count = self.get_connection().execute(
            "INSERT INTO documents_refs(id, rev, refs)
               SELECT id, rev, extract_refs(document_type, data)
               FROM documents_snapshots ds
               WHERE NOT EXISTS (
                 SELECT 1 FROM documents_refs dr WHERE dr.id = ds.id AND dr.rev = ds.rev COLLATE REV_CMP
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

    pub fn stage_document(
        &mut self,
        document: &mut Document,
        lock_key: Option<DocumentLockKey>,
    ) -> std::result::Result<(), StagingError> {
        log::debug!("Staging document {}", &document.id);

        let prev_document = self.get_document(&document.id)?;

        Validator::new(self).validate_staged(document, prev_document.as_ref())?;

        if prev_document.is_some() {
            log::debug!("Updating existing document {}", &document.id);
        } else {
            log::debug!("Creating new document {}", &document.id);
        }

        document.stage();
        document.updated_at = now();
        self.put_document(document, lock_key)?;

        self.register_event(BazaEvent::DocumentStaged {
            id: document.id.clone(),
        })?;

        log::info!("Saved document {}", document);

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
        document.stage();

        self.put_document(&document, None)?;

        log::info!("erased document {}", document);

        Ok(())
    }

    pub fn list_document_revisions(
        &self,
        min_rev: &Revision,
        skip_staged: bool,
    ) -> Result<Vec<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents_snapshots WHERE rev >= ?1 COLLATE REV_CMP")?;

        let mut rows = stmt.query([min_rev.serialize()])?;

        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            let document = utils::extract_document(row)?;

            // TODO optimize
            if skip_staged && document.is_staged() {
                continue;
            }

            // the query returns all the revisions that are bigger than, equal to or concurrent to min_rev
            // we don't need documents with revision equal to min_rev
            if document.rev == *min_rev {
                continue;
            }

            items.push(document);
        }

        Ok(items)
    }

    pub fn list_all_document_snapshots(&self) -> Result<Vec<Document>> {
        self.list_document_revisions(&Revision::initial(), false)
    }

    pub fn get_blob(&self, blob_id: &BLOBId) -> BLOB {
        BLOB::new(blob_id.clone(), &self.get_path_manager().data_dir)
    }

    pub fn get_existing_blob(&self, blob_id: &BLOBId) -> Result<Option<BLOB>> {
        let blob = self.get_blob(blob_id);

        Ok(blob.exists()?.then_some(blob))
    }

    pub fn get_local_blob_ids(&self) -> Result<HashSet<BLOBId>> {
        get_local_blob_ids(&self.get_path_manager().data_dir)
    }

    pub fn get_missing_blob_ids(&self) -> Result<HashSet<BLOBId>> {
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
            fs_tx.move_file(file_path, blob.file_path, false)?;
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

    fn remove_orphaned_blobs(&mut self) -> Result<()> {
        if self.has_staged_documents()? {
            log::warn!("there are staged documents, skipping");

            return Ok(());
        }

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

    pub(crate) fn apply_data_migrations(&self) -> Result<()> {
        let data_version = self.get_data_version()?;
        let schema = self.get_schema();
        let latest_data_version = schema.get_latest_data_version();
        let min_data_migration_version = schema.get_min_data_migration_version();

        ensure!(
            data_version <= latest_data_version,
            "data_version {} is bigger than latest data version {}",
            data_version,
            latest_data_version
        );

        ensure!(
            data_version >= min_data_migration_version - 1,
            "data_version {} is smaller than a minimum upgradable version {}",
            data_version,
            min_data_migration_version - 1
        );

        let migrations = schema
            .migrations
            .iter()
            .filter(|migration| migration.get_version() > data_version)
            .collect::<Vec<_>>();

        if migrations.is_empty() {
            log::debug!("no schema migrations to apply");

            return Ok(());
        }

        log::info!("{} schema migrations to apply", migrations.len());

        let mut stmt = self
            .get_connection()
            .prepare("SELECT rowid FROM documents_snapshots")?;

        let row_ids = stmt
            .query_and_then([], |row| row.get(0).context("failed to get arg 0"))
            .context("failed to query documents snapshots")?
            .collect::<Result<Vec<i64>>>()?;

        for migration in &migrations {
            let now = Instant::now();
            let mut rows_count = 0;
            for rowid in &row_ids {
                let document = self.get_document_by_rowid(rowid)?;
                let mut document = Cow::Borrowed(&document);

                migration.update(&mut document, self)?;

                // update document only if it has been mutated
                if let Cow::Owned(document) = document {
                    self.put_or_replace_document(&document, true)?;
                    rows_count += 1;
                }
            }

            let version = migration.get_version();
            self.set_data_version(version)?;

            log::info!(
                "Migrated {rows_count} rows in {} seconds to version {version}",
                now.elapsed().as_secs_f32(),
            );
        }

        log::info!("Finished data migration");

        Ok(())
    }

    pub fn list_document_backrefs(&self, id: &Id) -> Result<Vec<Document>> {
        let documents = self
            .list_documents(&Filter::all_backrefs(id.clone()))?
            .items;

        Ok(documents)
    }

    pub fn list_document_collections(&self, id: &Id) -> Result<Vec<Document>> {
        let documents = self
            .list_documents(&Filter::all_collections(id.clone()))?
            .items;

        Ok(documents)
    }

    pub fn list_staged_documents(&self) -> Result<Vec<Document>> {
        let documents = self.list_documents(&Filter::all_staged_documents())?.items;

        Ok(documents)
    }

    // FIXME pub fn get_blob_stream(&self, hash: &hash) -> Result<FileStream>
    // FIXME pub fn write_blob_stream(&self, hash: &hash, stream: FileStream) -> Result<()>

    pub fn commit_staged_documents(&mut self) -> Result<usize> {
        ensure!(
            self.list_document_locks()?.is_empty(),
            "There must be no pending locks"
        );

        let mut max_rev = self.get_db_rev()?;

        let instance_id = self.get_instance_id()?;
        let max_local_version = self.get_max_rev_version(&instance_id)?;

        max_rev.set_version(&instance_id, max_local_version + 1);

        let mut staged_documents = self.list_staged_documents()?;

        for document in &mut staged_documents {
            document.rev = max_rev.clone();

            self.put_document(document, None)?;

            if document.is_erased() {
                self.erase_document_history(&document.id, &max_rev)?;
            }
        }

        self.delete_local_staged_changes()?;
        self.remove_orphaned_blobs()?;

        self.register_event(BazaEvent::DocumentsCommitted {})?;

        log::info!("Committed {} documents", staged_documents.len());

        Ok(staged_documents.len())
    }

    pub fn update_document_collections(
        &mut self,
        document: &Document,
        collections: &Vec<Id>,
    ) -> Result<()> {
        let schema = self.get_schema();
        let document_expert = DocumentExpert::new(&schema);
        let mut old_collections = self.list_document_collections(&document.id)?;

        for old_collection in &mut old_collections {
            if !collections.contains(&old_collection.id) {
                document_expert.remove_document_from_collection(document, old_collection)?;
                self.stage_document(old_collection, None)?;
            }
        }

        let old_collections_ids = old_collections
            .iter()
            .map(|collection| &collection.id)
            .collect::<Vec<_>>();

        for collection_id in collections {
            if !old_collections_ids.contains(&collection_id) {
                let mut collection = self.must_get_document(collection_id)?;
                document_expert.add_document_to_collection(document, &mut collection)?;
                self.stage_document(&mut collection, None)?;
            }
        }

        Ok(())
    }
}

impl Drop for BazaConnection {
    fn drop(&mut self) {
        match self {
            BazaConnection::Transaction { completed, .. } => {
                if *completed {
                    return;
                }

                log::warn!("Transaction wasn't committed, rolling back");

                if let Err(err) = self.rollback() {
                    log::error!("Transaction rollback failed: {}", err);
                }
            }

            BazaConnection::ReadOnly { .. } => {}
        };
    }
}
