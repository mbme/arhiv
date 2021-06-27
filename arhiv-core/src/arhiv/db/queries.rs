use std::collections::HashSet;

use anyhow::*;
use rusqlite::{
    params, params_from_iter,
    types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
    Connection, OptionalExtension,
};
use serde::{de::DeserializeOwned, Serialize};

use rs_utils::log;

use super::dto::*;
use super::query_builder::QueryBuilder;
use super::utils;
use crate::entities::*;

pub trait Queries {
    fn get_connection(&self) -> &Connection;

    fn get_setting<T: Serialize + DeserializeOwned>(&self, setting: DBSetting<T>) -> Result<T> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT value FROM settings WHERE key = ?1")?;

        let value: String = stmt
            .query_row([setting.0], |row| row.get(0))
            .context(anyhow!("failed to read setting {}", setting.0))?;

        serde_json::from_str(&value).context(anyhow!("failed to parse setting {}", setting.0))
    }

    fn get_db_status(&self) -> Result<DbStatus> {
        Ok(DbStatus {
            arhiv_id: self.get_setting(SETTING_ARHIV_ID)?,
            is_prime: self.get_setting(SETTING_IS_PRIME)?,
            db_version: self.get_setting(SETTING_DB_VERSION)?,
            db_rev: self.get_db_rev()?,
            last_sync_time: self.get_setting(SETTING_LAST_SYNC_TIME)?,
        })
    }

    fn get_db_rev(&self) -> Result<Revision> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT IFNULL(MAX(rev), 0) FROM documents_snapshots")?;

        stmt.query_row([], |row| row.get(0))
            .context("failed to query for db rev")
    }

    fn count_documents(&self) -> Result<DocumentsCount> {
        // count documents
        // count attachments
        // count tombstones
        self.get_connection()
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN type != ?1 AND type != ?2 AND rev > 0                  THEN 1 ELSE 0 END), 0) AS documents_committed,
                    IFNULL(SUM(CASE WHEN type != ?1 AND type != ?2 AND rev = 0 AND prev_rev > 0 THEN 1 ELSE 0 END), 0) AS documents_updated,
                    IFNULL(SUM(CASE WHEN type != ?1 AND type != ?2 AND rev = 0 AND prev_rev = 0 THEN 1 ELSE 0 END), 0) AS documents_new,

                    IFNULL(SUM(CASE WHEN type  = ?1                AND rev > 0                  THEN 1 ELSE 0 END), 0) AS attachments_committed,
                    IFNULL(SUM(CASE WHEN type  = ?1                AND rev = 0 AND prev_rev > 0 THEN 1 ELSE 0 END), 0) AS attachments_updated,
                    IFNULL(SUM(CASE WHEN type  = ?1                AND rev = 0 AND prev_rev = 0 THEN 1 ELSE 0 END), 0) AS attachments_new,

                    IFNULL(SUM(CASE WHEN type  = ?2                AND rev > 0                  THEN 1 ELSE 0 END), 0) AS tombstones_committed,
                    IFNULL(SUM(CASE WHEN type  = ?2                AND rev = 0 AND prev_rev > 0 THEN 1 ELSE 0 END), 0) AS tombstones_updated,
                    IFNULL(SUM(CASE WHEN type  = ?2                AND rev = 0 AND prev_rev = 0 THEN 1 ELSE 0 END), 0) AS tombstones_new
                FROM documents",
                [ATTACHMENT_TYPE, TOMBSTONE_TYPE],
                |row| Ok(DocumentsCount {
                    documents_committed: row.get(0)?,
                    documents_updated: row.get(1)?,
                    documents_new: row.get(2)?,

                    attachments_committed: row.get(3)?,
                    attachments_updated: row.get(4)?,
                    attachments_new: row.get(5)?,

                    tombstones_committed: row.get(6)?,
                    tombstones_updated: row.get(7)?,
                    tombstones_new: row.get(8)?,
                }),
            )
            .context("Failed to count documents")
    }

    fn count_conflicts(&self) -> Result<u32> {
        self.get_connection()
            .query_row("SELECT COUNT(*) FROM conflicts", [], |row| row.get(0))
            .context("failed to count conflicts")
    }

    fn has_staged_documents(&self) -> Result<bool> {
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

    fn get_last_update_time(&self) -> Result<Timestamp> {
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

    fn list_documents(&self, filter: Filter) -> Result<ListPage<Document>> {
        let mut qb = QueryBuilder::new();

        qb.select("*", "documents");

        match filter.mode {
            Some(FilterMode::Staged) => {
                qb.where_condition("rev = 0");
            }
            Some(FilterMode::Archived) => {
                qb.where_condition("archived = true");
            }
            None => {
                qb.where_condition("archived = false");
            }
        }

        for matcher in filter.matchers {
            match matcher {
                Matcher::Field {
                    ref selector,
                    ref pattern,
                } => {
                    qb.where_condition(format!(
                        "json_extract(data, {}) = {}",
                        qb.param(selector),
                        qb.param(pattern)
                    ));
                }
                Matcher::Search { ref pattern } => {
                    qb.and_select(format!(
                        "calculate_search_score(type, data, {}) AS search_score",
                        qb.param(pattern)
                    ));
                    qb.where_condition("search_score > 0");

                    qb.order_by("search_score", false);
                }
                Matcher::Type { document_type } => {
                    qb.where_condition(format!("type = {}", qb.param(document_type)));
                }
            }
        }

        for order in filter.order {
            match order {
                OrderBy::UpdatedAt { asc } => {
                    qb.order_by("updated_at", asc);
                }
                OrderBy::Field { ref selector, asc } => {
                    qb.order_by(format!("json_extract(data, {})", qb.param(selector)), asc);
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
                            "CASE json_extract(data, {}) {} ELSE {} END",
                            qb.param(selector),
                            cases,
                            enum_order.len(),
                        ),
                        asc,
                    );
                }
            }
        }

        let mut page_size: i32 = -1;
        match (filter.page_size, filter.page_offset) {
            (None, None) => {}
            (page_size_opt, page_offset_opt) => {
                page_size = page_size_opt.map(|val| val as i32).unwrap_or(-1);

                // fetch (page_size + 1) items so that we know that there are more items than page_size
                if page_size > -1 {
                    page_size += 1
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

    fn get_new_snapshots_since(&self, min_rev: &Revision) -> Result<Vec<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents_snapshots WHERE rev >= ?1")?;

        let mut rows = stmt
            .query_and_then([min_rev], utils::extract_document)
            .context(anyhow!("Failed to get new snapshots since {}", min_rev))?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next() {
            documents.push(row?);
        }

        Ok(documents)
    }

    fn get_document(&self, id: &Id) -> Result<Option<Document>> {
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

    fn get_attachment_ids(&self) -> Result<HashSet<Id>> {
        let mut stmt = self
            .get_connection()
            .prepare("SELECT id FROM documents WHERE type = ?1")?;

        let mut rows = stmt.query_and_then([ATTACHMENT_TYPE], |row| row.get("id"))?;

        let mut result = HashSet::new();
        while let Some(entry) = rows.next() {
            let id: String = entry?;
            result.insert(id.into());
        }

        Ok(result)
    }

    fn has_snapshot(&self, id: &SnapshotId) -> Result<bool> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT true FROM documents_snapshots WHERE snapshot_id = ?1")?;

        stmt.query_row([id], |_row| Ok(true))
            .optional()
            .context(anyhow!("Failed to check for snapshot {}", &id))
            .map(|value| value.unwrap_or(false))
    }

    fn get_last_snapshot(&self, id: &Id) -> Result<Option<Document>> {
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
}

pub trait MutableQueries: Queries {
    fn set_setting<T: Serialize + DeserializeOwned>(
        &self,
        setting: DBSetting<T>,
        value: T,
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

    fn put_document(&self, document: &Document) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(&format!(
            "INSERT {} INTO documents_snapshots
            (id, rev, prev_rev, snapshot_id, type, created_at, updated_at, archived, refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
            document.snapshot_id,
            document.document_type,
            document.created_at,
            document.updated_at,
            document.archived,
            utils::serialize_refs(&document.refs)?,
            document.data,
        ])
        .context(anyhow!("Failed to put document {}", &document.id))?;

        Ok(())
    }

    // delete all document versions except the latest one
    fn erase_document_history(&self, id: &Id) -> Result<()> {
        let rows_count = self.get_connection().execute(
            "DELETE FROM documents_snapshots
             WHERE id = ?1 AND rev <> (SELECT MAX(rev) FROM documents_snapshots WHERE id = ?1)",
            [id],
        )?;

        log::debug!("deleted {} rows of history for document {}", rows_count, id);

        Ok(())
    }

    fn delete_unused_local_attachments(&self) -> Result<()> {
        // find all documents which
        // 1. are staged (rev = 0)
        // 2. are new (prev_rev = 0)
        // 3. are of type "attachment"
        // 4. aren't referenced by staged documents
        let rows_count = self.get_connection()
            .prepare_cached(
                "WITH new_ids_in_use AS (SELECT DISTINCT json_each.value FROM documents_snapshots, json_each(refs) WHERE rev = 0)
                DELETE FROM documents_snapshots WHERE rev = 0
                                                AND prev_rev = 0
                                                AND type = ?1
                                                AND id NOT IN new_ids_in_use"
            )?.execute([ATTACHMENT_TYPE])
            .context("Failed to delete unused local attachments")?;

        log::debug!("deleted {} unused local attachments", rows_count);

        Ok(())
    }

    fn delete_local_staged_changes(&self) -> Result<()> {
        self.get_connection()
            .execute("DELETE FROM documents_snapshots WHERE rev = 0", [])?;

        Ok(())
    }
}

impl FromSql for Revision {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_i64()
            .map(|value| Revision::from_value(value as u32))
    }
}

impl ToSql for Revision {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0))
    }
}

impl FromSql for Id {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(Id::from)
    }
}

impl ToSql for Id {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self as &str))
    }
}

impl FromSql for SnapshotId {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(SnapshotId::from)
    }
}

impl ToSql for SnapshotId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self as &str))
    }
}
