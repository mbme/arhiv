use super::query_params::*;
use super::utils;
use crate::entities::*;
use anyhow::*;
use rs_utils::fuzzy_match;
use rusqlite::functions::FunctionFlags;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{params, Connection, OptionalExtension, NO_PARAMS};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub trait Queries {
    fn get_connection(&self) -> &Connection;

    fn get_setting<S: Into<String>>(&self, key: S) -> Result<Option<String>> {
        self.get_connection()
            .query_row(
                "SELECT value FROM settings WHERE key=?",
                params![key.into()],
                |row| row.get(0),
            )
            .optional()
            .context("failed to get setting")
    }

    fn is_prime(&self) -> Result<bool> {
        self.get_setting("is_prime")
            .map(|value| value.unwrap_or("".to_string()) == "true")
    }

    fn get_rev(&self) -> Result<Revision> {
        self.get_connection()
            .query_row(
                "SELECT IFNULL(MAX(rev), 0) FROM (
                    SELECT MAX(rev) as rev FROM documents
                        UNION ALL
                    SELECT MAX(rev) as rev FROM attachments)",
                NO_PARAMS,
                |row| row.get(0).into(),
            )
            .context("failed to get max rev")
    }

    fn count_documents(&self) -> Result<(u32, u32)> {
        self.get_connection()
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN rev > 0 THEN 1 ELSE 0 END), 0) AS committed,
                    IFNULL(SUM(CASE WHEN rev = 0 THEN 1 ELSE 0 END), 0) AS staged
                FROM documents",
                NO_PARAMS,
                |row| Ok((row.get_unwrap(0), row.get_unwrap(1))),
            )
            .context("Failed to count documents")
    }

    fn count_attachments(&self) -> Result<(u32, u32)> {
        self.get_connection()
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN rev > 0 THEN 1 ELSE 0 END), 0) AS committed,
                    IFNULL(SUM(CASE WHEN rev = 0 THEN 1 ELSE 0 END), 0) AS staged
                FROM attachments",
                NO_PARAMS,
                |row| Ok((row.get_unwrap(0), row.get_unwrap(1))),
            )
            .context("Failed to count attachments")
    }

    fn list_documents(&self, filter: DocumentFilter) -> Result<ListPage<Document>> {
        let mut query: Vec<String> = vec!["SELECT * FROM documents WHERE true".to_string()];
        let mut params = Params::new();

        if filter.only_staged.unwrap_or(false) {
            query.push("AND rev = 0".to_string())
        }

        for (i, matcher) in filter.matchers.into_iter().enumerate() {
            let matcher_selector_var = format!(":matcher_selector_{}", i);
            let matcher_pattern_var = format!(":matcher_pattern_{}", i);

            if matcher.fuzzy {
                self.init_fuzzy_search()?;

                query.push(format!(
                    "AND fuzzySearch(json_extract(data, {}), {})",
                    matcher_selector_var, matcher_pattern_var,
                ));
            } else {
                query.push(format!(
                    "AND json_extract(data, {}) = {}",
                    matcher_selector_var, matcher_pattern_var,
                ));
            }

            params.insert(&matcher_selector_var, Rc::new(matcher.selector));
            params.insert(&matcher_pattern_var, Rc::new(matcher.pattern));
        }

        query.push("AND archived = :archived".to_string());
        params.insert(":archived", Rc::new(filter.archived.unwrap_or(false)));

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
        log::debug!("list_documents: {}", &query);
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

        let mut rows = stmt.query(params![min_rev])?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(utils::extract_document(row)?);
        }

        Ok(documents)
    }

    fn list_attachments(&self, filter: AttachmentFilter) -> Result<ListPage<Attachment>> {
        let mut query = vec!["SELECT * FROM attachments WHERE true"];
        let mut params = Params::new();

        let mut page_size: i32 = -1;
        match (filter.page_size, filter.page_offset) {
            (None, None) => {}
            (page_size_opt, page_offset_opt) => {
                page_size = page_size_opt.map(|val| val as i32).unwrap_or(-1);

                // fetch (page_size + 1) items so that we know that there are more items than page_size
                if page_size > -1 {
                    page_size += 1
                }

                query.push("LIMIT :limit");
                params.insert(":limit", Rc::new(page_size));

                let page_offset = page_offset_opt.unwrap_or(0);
                query.push("OFFSET :offset");
                params.insert(":offset", Rc::new(page_offset));
            }
        }

        let query = query.join(" ");
        log::debug!("list_attachments: {}", &query);
        let mut stmt = self.get_connection().prepare_cached(&query)?;

        let mut rows = stmt.query_named(&params.get())?;

        let mut items = Vec::new();
        let mut has_more = false;
        while let Some(row) = rows.next()? {
            if page_size > -1 && items.len() as i32 == page_size - 1 {
                has_more = true;
                break; // due to break we ignore last item
            }

            items.push(utils::extract_attachment(row)?);
        }

        Ok(ListPage { items, has_more })
    }

    fn get_attachments_since(&self, min_rev: &Revision) -> Result<Vec<Attachment>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM attachments WHERE rev >= ?1")?;

        let mut rows = stmt.query(vec![min_rev])?;

        let mut attachments = Vec::new();
        while let Some(row) = rows.next()? {
            attachments.push(utils::extract_attachment(row)?);
        }

        Ok(attachments)
    }

    fn get_staged_attachments(&self) -> Result<Vec<Attachment>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM attachments WHERE rev = 0")?;

        let mut rows = stmt.query(NO_PARAMS)?;

        let mut attachments = Vec::new();
        while let Some(row) = rows.next()? {
            attachments.push(utils::extract_attachment(row)?);
        }

        Ok(attachments)
    }

    fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents WHERE id = ?1")?;

        let mut rows = stmt.query_and_then(params![id], utils::extract_document)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM attachments WHERE id = ?1")?;

        let mut rows = stmt.query_and_then(params![id], utils::extract_attachment)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    fn get_changeset(&self) -> Result<Changeset> {
        let documents = self.list_documents(DOCUMENT_FILTER_STAGED)?.items;

        let ids_in_use: HashSet<Id> = documents
            .iter()
            .map(|document| document.refs.clone())
            .flatten()
            .collect();

        let attachments = self
            .get_staged_attachments()?
            .into_iter()
            // ignore unused local attachments
            .filter(|attachment| ids_in_use.contains(&attachment.id))
            .collect();

        let changeset = Changeset {
            base_rev: self.get_rev()?,
            documents,
            attachments,
        };
        log::debug!("prepared a changeset {}", changeset);

        Ok(changeset)
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

    fn set_setting<S: Into<String>>(&self, key: S, value: Option<String>) -> Result<()> {
        self.get_connection()
            .execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
                params![key.into(), value],
            )
            .context("failed to set setting")?;

        Ok(())
    }

    fn put_document(&self, document: &Document) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT OR REPLACE INTO documents
            (id, rev, created_at, updated_at, archived, refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            document.id,
            document.rev,
            document.created_at,
            document.updated_at,
            document.archived,
            utils::serialize_refs(&document.refs)?,
            document.data,
        ])?;

        Ok(())
    }

    fn put_document_history(&self, document: &Document) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT INTO documents_history
            (id, rev, created_at, updated_at, archived, refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            document.id,
            document.rev,
            document.created_at,
            document.updated_at,
            document.archived,
            utils::serialize_refs(&document.refs)?,
            document.data,
        ])?;

        Ok(())
    }

    fn put_attachment(&self, attachment: &Attachment, allow_update: bool) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(&format!(
            "INSERT {} INTO attachments
            (id, rev, hash, created_at, filename, archived)
            VALUES (?, ?, ?, ?, ?, ?)",
            if allow_update { "OR REPLACE" } else { "" }
        ))?;

        stmt.execute(params![
            attachment.id,
            attachment.rev,
            attachment.hash,
            attachment.created_at,
            attachment.filename,
            attachment.archived,
        ])?;

        Ok(())
    }
}

struct Params {
    params: HashMap<String, Rc<dyn ToSql>>,
}

impl Params {
    pub fn new() -> Self {
        Params {
            params: HashMap::new(),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, key: S, value: Rc<dyn ToSql>) {
        self.params.insert(key.into(), value);
    }

    pub fn get(&self) -> Vec<(&str, &dyn ToSql)> {
        let mut params: Vec<(&str, &dyn ToSql)> = vec![];
        for (key, value) in self.params.iter() {
            params.push((key, value.as_ref()));
        }

        params
    }
}

impl FromSql for Revision {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().map(|value| (value as u32).into())
    }
}

impl ToSql for Revision {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0))
    }
}

impl FromSql for Id {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(|value| value.to_string().into())
    }
}

impl ToSql for Id {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let value: &str = &self.0;

        Ok(ToSqlOutput::from(value))
    }
}
