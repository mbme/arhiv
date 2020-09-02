use super::query_params::*;
use super::utils;
use crate::entities::*;
use anyhow::*;
use rs_utils::fuzzy_match;
use rusqlite::functions::FunctionFlags;
use rusqlite::Error as RusqliteError;
use rusqlite::{params, Connection, OptionalExtension, ToSql, NO_PARAMS};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

struct Params {
    params: HashMap<&'static str, Rc<dyn ToSql>>,
}

impl Params {
    pub fn new() -> Self {
        Params {
            params: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &'static str, value: Rc<dyn ToSql>) {
        self.params.insert(key, value);
    }

    pub fn get(&self) -> Vec<(&str, &dyn ToSql)> {
        let mut params: Vec<(&str, &dyn ToSql)> = vec![];
        for (key, value) in self.params.iter() {
            params.push((key, value.as_ref()));
        }

        params
    }
}

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
                |row| row.get(0),
            )
            .context("failed to get max rev")
    }

    fn count_documents(&self) -> Result<(u32, u32)> {
        self.get_connection()
            .query_row(
                "SELECT
                    IFNULL(SUM(CASE WHEN staged = false THEN 1 ELSE 0 END), 0) AS committed,
                    IFNULL(SUM(CASE WHEN staged = true THEN 1 ELSE 0 END), 0) AS staged
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

    fn list_documents(&self, filter: DocumentFilter) -> Result<Vec<Document>> {
        let mut query = vec!["SELECT * FROM documents WHERE true"];
        let mut params = Params::new();

        if filter.only_staged.unwrap_or(false) {
            query.push("AND staged = true")
        }

        if let Some(document_type) = filter.document_type {
            query.push("AND type = :type");
            params.insert(":type", Rc::new(document_type));
        }

        if let Some(matcher) = filter.matcher {
            self.init_fuzzy_search()?;

            query.push("AND fuzzySearch(json_extract(data, :matcher_selector), :matcher_pattern)");
            params.insert(":matcher_selector", Rc::new(matcher.selector));
            params.insert(":matcher_pattern", Rc::new(matcher.pattern));
        }

        if filter.skip_archived.unwrap_or(false) {
            query.push("AND archived = false");
        }

        query.push("GROUP BY id HAVING staged = true OR staged = false");

        match (filter.page_size, filter.page_offset) {
            (None, None) => {}
            (page_size, page_offset) => {
                let page_size = page_size
                    .map(|val| val.to_string())
                    .unwrap_or("-1".to_string());

                query.push("LIMIT :limit");
                params.insert(":limit", Rc::new(page_size));

                let page_offset = page_offset.unwrap_or(0);
                query.push("OFFSET :offset");
                params.insert(":offset", Rc::new(page_offset));
            }
        }

        let query = query.join(" ");
        log::trace!("list_documents: {}", &query);
        let mut stmt = self.get_connection().prepare_cached(&query)?;

        let mut rows = stmt.query_named(&params.get())?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(utils::extract_document(row)?);
        }

        Ok(documents)
    }

    fn get_documents_since(&self, min_rev: Revision) -> Result<Vec<Document>> {
        let mut stmt = self.get_connection().prepare_cached(
            "SELECT * FROM documents_history WHERE rev >= ?1 GROUP BY id HAVING MAX(rev)",
        )?;

        let mut rows = stmt.query(params![min_rev])?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(utils::extract_document(row)?);
        }

        Ok(documents)
    }

    fn list_attachments(&self, filter: AttachmentFilter) -> Result<Vec<Attachment>> {
        let mut query = vec!["SELECT * FROM attachments WHERE true"];
        let mut params = Params::new();

        if let Some(pattern) = filter.pattern {
            self.init_fuzzy_search()?;

            query.push("AND fuzzySearch(filename, :matcher_pattern)");
            params.insert(":pattern", Rc::new(pattern));
        }

        match (filter.page_size, filter.page_offset) {
            (None, None) => {}
            (page_size, page_offset) => {
                let page_size = page_size
                    .map(|val| val.to_string())
                    .unwrap_or("-1".to_string());

                query.push("LIMIT :limit");
                params.insert(":limit", Rc::new(page_size));

                let page_offset = page_offset.unwrap_or(0);
                query.push("OFFSET :offset");
                params.insert(":offset", Rc::new(page_offset));
            }
        }

        let query = query.join(" ");
        log::trace!("list_attachments: {}", &query);
        let mut stmt = self.get_connection().prepare_cached(&query)?;

        let mut rows = stmt.query_named(&params.get())?;

        let mut attachments = Vec::new();
        while let Some(row) = rows.next()? {
            attachments.push(utils::extract_attachment(row)?);
        }

        Ok(attachments)
    }

    fn get_attachments_since(&self, min_rev: Revision) -> Result<Vec<Attachment>> {
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
            .prepare_cached("SELECT * FROM documents WHERE id = ?1 GROUP BY id HAVING staged = true OR staged = false")?;

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
        let documents = self.list_documents(DOCUMENT_FILTER_STAGED)?;

        let attachments_in_use: HashSet<String> = documents
            .iter()
            .map(|document| document.attachment_refs.clone())
            .flatten()
            .collect();

        let attachments = self
            .get_staged_attachments()?
            .into_iter()
            // ignore unused local attachments
            .filter(|attachment| attachments_in_use.contains(&attachment.id))
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

    fn put_document(&self, document: &Document, staged: bool) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT OR REPLACE INTO documents
            (staged, id, rev, created_at, updated_at, archived, type, refs, attachment_refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            staged,
            document.id,
            document.rev,
            document.created_at,
            document.updated_at,
            document.archived,
            document.document_type,
            utils::serialize_refs(&document.refs)?,
            utils::serialize_refs(&document.attachment_refs)?,
            document.data,
        ])?;

        Ok(())
    }

    fn put_document_history(&self, document: &Document) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT INTO documents_history
            (id, rev, created_at, updated_at, archived, type, refs, attachment_refs, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            document.id,
            document.rev,
            document.created_at,
            document.updated_at,
            document.archived,
            document.document_type,
            utils::serialize_refs(&document.refs)?,
            utils::serialize_refs(&document.attachment_refs)?,
            document.data,
        ])?;

        Ok(())
    }

    fn delete_staged_documents(&self) -> Result<()> {
        self.get_connection()
            .execute("DELETE FROM documents WHERE staged = true", NO_PARAMS)?;

        Ok(())
    }

    fn put_attachment(&self, attachment: &Attachment) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT OR REPLACE INTO attachments
            (id, rev, hash, created_at, filename, archived)
            VALUES (?, ?, ?, ?, ?, ?)",
        )?;

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
