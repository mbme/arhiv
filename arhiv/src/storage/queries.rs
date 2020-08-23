use super::query_params::*;
use super::utils;
use crate::entities::*;
use crate::utils::fuzzy_match;
use anyhow::*;
use rusqlite::functions::FunctionFlags;
use rusqlite::Error as RusqliteError;
use rusqlite::{params, Connection, ToSql, NO_PARAMS};
use std::collections::HashMap;
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

    fn get_rev(&self) -> Result<Revision> {
        let conn = self.get_connection();

        let rev = conn.query_row(
            "SELECT IFNULL(MAX(rev), 0) FROM (SELECT rev FROM documents UNION ALL SELECT rev FROM attachments)",
            NO_PARAMS,
            |row| row.get(0),
        )?;

        Ok(rev)
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

    fn has_staged_changes(&self) -> Result<bool> {
        let (_, staged_documents) = self.count_documents()?;
        if staged_documents > 0 {
            return Ok(true);
        }

        let (_, staged_attachments) = self.count_attachments()?;

        Ok(staged_attachments > 0)
    }

    fn get_documents(&self, min_rev: Revision, filter: DocumentFilter) -> Result<Vec<Document>> {
        let mut query = vec!["SELECT * FROM documents WHERE rev >= :min_rev"];

        let mut params = Params::new();
        params.insert(":min_rev", Rc::new(min_rev));

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

        // local documents with rev === 0 have higher priority
        query.push("GROUP BY id ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END)");

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
        log::trace!("get_documents: {}", &query);
        let mut stmt = self.get_connection().prepare_cached(&query)?;

        let mut rows = stmt.query_named(&params.get())?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(utils::extract_document(row)?);
        }

        Ok(documents)
    }

    fn get_staged_documents(&self) -> Result<Vec<Document>> {
        let mut stmt = self
            .get_connection()
            .prepare_cached("SELECT * FROM documents WHERE rev = 0")?;

        let mut rows = stmt.query(NO_PARAMS)?;

        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(utils::extract_document(row)?);
        }

        Ok(documents)
    }

    fn get_attachments(
        &self,
        min_rev: Revision,
        filter: AttachmentFilter,
    ) -> Result<Vec<Attachment>> {
        let mut query = vec!["SELECT * FROM attachments WHERE rev >= :min_rev"];

        let mut params = Params::new();
        params.insert(":min_rev", Rc::new(min_rev));

        if let Some(pattern) = filter.pattern {
            self.init_fuzzy_search()?;

            query.push("AND fuzzySearch(filename, :matcher_pattern)");
            params.insert(":pattern", Rc::new(pattern));
        }

        // local attachments with rev === 0 have higher priority
        query.push("GROUP BY id ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END)");

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
        log::trace!("get_attachments: {}", &query);
        let mut stmt = self.get_connection().prepare_cached(&query)?;

        let mut rows = stmt.query_named(&params.get())?;

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
        // FIXME does this query correctly return max revision?
        let mut stmt = self.get_connection().prepare_cached("SELECT * FROM documents WHERE id = ?1 ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END) LIMIT 1")?;

        let mut rows = stmt.query_and_then(params![id], utils::extract_document)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    fn get_attachment(&self, id: &Id, only_committed: bool) -> Result<Option<Attachment>> {
        // FIXME does this query correctly return max revision?
        let mut stmt = self.get_connection().prepare_cached("SELECT * FROM attachments WHERE id = ?1 ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END) LIMIT 1")?;

        let mut rows = stmt.query_and_then(params![id], utils::extract_attachment)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    fn put_document(&self, document: &Document) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT OR REPLACE INTO documents
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

    fn put_attachment(&self, attachment: &Attachment) -> Result<()> {
        let mut stmt = self.get_connection().prepare_cached(
            "INSERT OR REPLACE INTO attachments
        (id, rev, created_at, filename)
        VALUES (?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            attachment.id,
            attachment.rev,
            attachment.created_at,
            attachment.filename,
        ])?;

        Ok(())
    }

    fn delete_staged_documents(&self) -> Result<()> {
        let rows = self
            .get_connection()
            .execute("DELETE FROM documents WHERE rev = 0", NO_PARAMS)?;

        log::debug!("deleted {} staged documents", rows);

        Ok(())
    }

    fn delete_staged_attachments(&self) -> Result<()> {
        let rows = self
            .get_connection()
            .execute("DELETE FROM attachments WHERE rev = 0", NO_PARAMS)?;

        log::debug!("deleted {} staged attachments", rows);

        Ok(())
    }

    fn get_changeset(&self) -> Result<Changeset> {
        let changeset = Changeset {
            base_rev: self.get_rev()?,
            documents: self.get_staged_documents()?,
            attachments: self.get_staged_attachments()?, // FIXME ignore unused local attachments
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
