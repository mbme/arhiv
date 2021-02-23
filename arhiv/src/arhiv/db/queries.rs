use std::rc::Rc;

use super::dto::*;
use super::utils;
use crate::entities::*;
use anyhow::*;
use rs_utils::{fuzzy_match, log::debug};
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{functions::FunctionFlags, NO_PARAMS};
use rusqlite::{params, Connection, OptionalExtension};

const DB_STATUS_KEY: &'static str = "status";

pub trait Queries {
    fn get_connection(&self) -> &Connection;

    fn get_db_status(&self) -> Result<DbStatus> {
        self.get_connection()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![DB_STATUS_KEY],
                |row| {
                    let value: String = row.get_unwrap(0);

                    Ok(serde_json::from_str(&value).expect("must parse DbStatus"))
                },
            )
            .context("failed to read DbStatus")
    }

    fn count_documents(&self, last_sync_time: &Timestamp) -> Result<DocumentsCount> {
        self.get_connection()
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN type != ?1 AND rev > 0                      THEN 1 ELSE 0 END), 0) AS documents_committed,
                    IFNULL(SUM(CASE WHEN type != ?1 AND rev = 0 AND created_at <= ?2 THEN 1 ELSE 0 END), 0) AS documents_updated,
                    IFNULL(SUM(CASE WHEN type != ?1 AND rev = 0 AND created_at  > ?2 THEN 1 ELSE 0 END), 0) AS documents_new,
                    IFNULL(SUM(CASE WHEN type  = ?1 AND rev > 0                      THEN 1 ELSE 0 END), 0) AS attachments_committed,
                    IFNULL(SUM(CASE WHEN type  = ?1 AND rev = 0 AND created_at <= ?2 THEN 1 ELSE 0 END), 0) AS attachments_updated,
                    IFNULL(SUM(CASE WHEN type  = ?1 AND rev = 0 AND created_at  > ?2 THEN 1 ELSE 0 END), 0) AS attachments_new
                FROM documents",
                vec![ATTACHMENT_TYPE.to_sql()?, last_sync_time.to_sql()?],
                |row| Ok(DocumentsCount {
                    documents_committed: row.get_unwrap(0),
                    documents_updated: row.get_unwrap(1),
                    documents_new: row.get_unwrap(2),
                    attachments_committed: row.get_unwrap(3),
                    attachments_updated: row.get_unwrap(4),
                    attachments_new: row.get_unwrap(5),
                }),
            )
            .context("Failed to count documents")
    }

    fn has_staged_documents(&self) -> Result<bool> {
        self.get_connection()
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE rev = 0",
                NO_PARAMS,
                |row| Ok(row.get_unwrap::<_, u32>(0) > 0),
            )
            .context("Failed to check for staged documents")
    }

    fn get_last_update_time(&self) -> Result<Timestamp> {
        let result: Option<Timestamp> = self
            .get_connection() // FIXME check if this ordering actually works
            .query_row(
                "SELECT updated_at FROM documents ORDER BY updated_at DESC LIMIT 1",
                NO_PARAMS,
                |row| Ok(row.get_unwrap(0)),
            )
            .optional()
            .context("Failed to get last update time")?;

        Ok(result.unwrap_or(chrono::MIN_DATETIME))
    }

    fn list_documents(&self, filter: Filter) -> Result<ListPage<Document>> {
        let mut query: Vec<String> = vec!["SELECT * FROM documents WHERE true".to_string()];
        let mut params = utils::Params::new();

        match filter.mode {
            Some(FilterMode::Staged) => {
                query.push("AND rev = 0".to_string());
            }
            Some(FilterMode::Archived) => {
                query.push("AND archived = true".to_string());
            }
            None => {
                query.push("AND archived = false".to_string());
            }
        }

        for (i, matcher) in filter.matchers.into_iter().enumerate() {
            match matcher {
                Matcher::Field { selector, pattern } => {
                    let matcher_selector_var = format!(":matcher_selector_{}", i);
                    let matcher_pattern_var = format!(":matcher_pattern_{}", i);

                    query.push(format!(
                        "AND json_extract(data, {}) = {}",
                        matcher_selector_var, matcher_pattern_var,
                    ));

                    params.insert(&matcher_selector_var, Rc::new(selector));
                    params.insert(&matcher_pattern_var, Rc::new(pattern));
                }
                Matcher::FuzzyField { selector, pattern } => {
                    let matcher_selector_var = format!(":matcher_selector_{}", i);
                    let matcher_pattern_var = format!(":matcher_pattern_{}", i);

                    self.init_fuzzy_search()?;

                    query.push(format!(
                        "AND fuzzySearch(json_extract(data, {}), {})",
                        matcher_selector_var, matcher_pattern_var,
                    ));
                    params.insert(&matcher_selector_var, Rc::new(selector));
                    params.insert(&matcher_pattern_var, Rc::new(pattern));
                }
                Matcher::Type { document_type } => {
                    let matcher_type_var = format!(":matcher_type_{}", i);

                    query.push(format!("AND type = {}", matcher_type_var));

                    params.insert(&matcher_type_var, Rc::new(document_type));
                }
            }
        }

        if !filter.order.is_empty() {
            query.push("ORDER BY".to_string());

            let mut order_query = vec![];

            for (i, order) in filter.order.into_iter().enumerate() {
                match order {
                    OrderBy::UpdatedAt { asc } => {
                        order_query
                            .push(format!("updated_at {}", if asc { "ASC" } else { "DESC" }));
                    }
                    OrderBy::Field { selector, asc } => {
                        let selector_var = format!(":order_selector_{}", i);

                        order_query.push(format!(
                            "json_extract(data, {}) {}",
                            selector_var,
                            if asc { "ASC" } else { "DESC" }
                        ));

                        params.insert(&selector_var, Rc::new(selector))
                    }
                    OrderBy::EnumField {
                        selector,
                        asc,
                        enum_order,
                    } => {
                        let selector_var = format!(":order_selector_{}", i);

                        // TODO use variables instead of string interp
                        let cases = enum_order
                            .iter()
                            .enumerate()
                            .map(|(j, item)| format!("WHEN '{}' THEN {}", item, j))
                            .collect::<Vec<String>>()
                            .join(" ");

                        order_query.push(format!(
                            "CASE json_extract(data, {}) {} ELSE {} END {}",
                            selector_var,
                            cases,
                            enum_order.len(),
                            if asc { "ASC" } else { "DESC" }
                        ));

                        params.insert(&selector_var, Rc::new(selector))
                    }
                }
            }

            query.push(order_query.join(", "));
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

                query.push("LIMIT :limit".to_string());
                params.insert(":limit", Rc::new(page_size));

                let page_offset = page_offset_opt.unwrap_or(0);
                query.push("OFFSET :offset".to_string());
                params.insert(":offset", Rc::new(page_offset));
            }
        }

        let query = query.join(" ");
        debug!("list_documents: {}", &query);
        let mut stmt = self.get_connection().prepare_cached(&query)?;

        let mut rows = stmt.query_named(&params.get())?;

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

    fn get_documents_since(&self, min_rev: &Revision) -> Result<Vec<Document>> {
        let mut stmt = self.get_connection().prepare_cached(
            "SELECT * FROM documents_history WHERE rev >= ?1 GROUP BY id HAVING rev = MAX(rev)",
        )?;

        let mut rows = stmt
            .query(params![min_rev])
            .context(anyhow!("Failed to get documents since {}", min_rev))?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(utils::extract_document(row)?);
        }

        Ok(documents)
    }

    fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents WHERE id = ?1")?;

        let mut rows = stmt
            .query_and_then(params![id], utils::extract_document)
            .context(anyhow!("Failed to get document {}", &id))?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    fn init_fuzzy_search(&self) -> Result<()> {
        self.get_connection()
            .create_scalar_function(
                "fuzzySearch",
                2,
                FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
                move |ctx| {
                    use rusqlite::Error as RusqliteError;
                    assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");

                    let haystack = ctx
                        .get_raw(0)
                        .as_str()
                        .map_err(|e| RusqliteError::UserFunctionError(e.into()))?;

                    let needle = ctx
                        .get_raw(1)
                        .as_str()
                        .map_err(|e| RusqliteError::UserFunctionError(e.into()))?;

                    Ok(fuzzy_match(needle, haystack))
                },
            )
            .context(anyhow!("Failed to define fuzzySearch function"))
    }
}

pub trait MutableQueries: Queries {
    fn create_tables(&self) -> Result<()> {
        self.get_connection()
            .execute_batch(include_str!("./schema.sql"))?;

        Ok(())
    }

    fn put_db_status(&self, status: DbStatus) -> Result<()> {
        self.get_connection()
            .execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                params![DB_STATUS_KEY, serde_json::to_string(&status)?],
            )
            .context("failed to save DbStatus")?;

        Ok(())
    }

    fn put_document(&self, document: &Document) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT OR REPLACE INTO documents
            (id, rev, type, created_at, updated_at, archived, refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            document.id,
            document.rev,
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

    fn put_document_history(&self, document: &Document, base_rev: &Revision) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT INTO documents_history
            (id, rev, base_rev, type, created_at, updated_at, archived, refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            document.id,
            document.rev,
            base_rev,
            document.document_type,
            document.created_at,
            document.updated_at,
            document.archived,
            utils::serialize_refs(&document.refs)?,
            document.data,
        ])
        .context(anyhow!(
            "Failed to put document into history {} rev {}",
            &document.id,
            &document.rev
        ))?;

        Ok(())
    }

    fn delete_document(&self, id: &Id) -> Result<()> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("DELETE FROM documents WHERE id = ?")?;

        stmt.execute(params![id])
            .context(anyhow!("Failed to delete document {}", id))?;

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
        value
            .as_str()
            .map(|value| Id::from_string(value.to_string()))
    }
}

impl ToSql for Id {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_ref()))
    }
}
