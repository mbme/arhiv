use super::query_params::*;
use super::utils;
use crate::entities::*;
use anyhow::*;
use rusqlite::{params, Connection, ToSql, NO_PARAMS};

pub fn get_rev(conn: &Connection) -> Result<Revision> {
    let rev = conn.query_row(
        "SELECT IFNULL(MAX(rev), 0) FROM (SELECT rev FROM documents UNION ALL SELECT rev FROM attachments)",
        NO_PARAMS,
        |row| row.get(0),
    )?;

    Ok(rev)
}

pub fn has_staged_changes(conn: &Connection) -> Result<bool> {
    let documents_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM documents WHERE rev = 0",
        NO_PARAMS,
        |row| row.get(0),
    )?;

    if documents_count > 0 {
        return Ok(true);
    }

    let attachments_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM attachments WHERE rev = 0",
        NO_PARAMS,
        |row| row.get(0),
    )?;

    Ok(attachments_count > 0)
}

pub fn get_documents(
    conn: &Connection,
    min_rev: Revision,
    filter: QueryFilter,
) -> Result<Vec<Document>> {
    let mut query = vec!["SELECT * FROM documents WHERE rev >= :min_rev"];
    let mut params: Vec<(&str, &dyn ToSql)> = vec![("min_rev", &min_rev)];

    if let Some(ref document_type) = filter.document_type {
        query.push("AND type = :type");
        params.push(("type", document_type));
    }

    query.push("GROUP BY id HAVING max(rev) ORDER BY rev DESC");

    if let Some(ref page) = filter.page {
        query.push("LIMIT :limit OFFSET :offset");
        params.push(("limit", &page.size));
        params.push(("offset", &page.offset));
    }

    let mut stmt = conn.prepare_cached(&query.join(" "))?;

    let row = stmt.query_named(&params)?;

    utils::extract_documents(row)
}

pub fn get_staged_documents(conn: &Connection) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM documents WHERE rev = 0")?;

    let row = stmt.query(NO_PARAMS)?;

    utils::extract_documents(row)
}

pub fn get_all_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM attachments ORDER BY rev DESC")?;

    let row = stmt.query(NO_PARAMS)?;

    utils::extract_attachments(row)
}

pub fn get_commited_attachments_with_rev(
    conn: &Connection,
    min_rev: Revision,
) -> Result<Vec<Attachment>> {
    let mut stmt =
        conn.prepare_cached("SELECT * FROM attachments WHERE rev >= ?1 ORDER BY rev DESC")?;

    let row = stmt.query(params![min_rev])?;

    utils::extract_attachments(row)
}

pub fn get_commited_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    get_commited_attachments_with_rev(conn, 1)
}

pub fn get_staged_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    let mut stmt =
        conn.prepare_cached("SELECT * FROM attachments WHERE rev = 0 ORDER BY rev DESC")?;

    let row = stmt.query(NO_PARAMS)?;

    utils::extract_attachments(row)
}

pub fn get_document(conn: &Connection, id: &Id, mode: QueryMode) -> Result<Option<Document>> {
    let mut stmt = conn.prepare_cached({
        match mode {
            QueryMode::All => {
                "SELECT * FROM documents WHERE id = ?1 GROUP BY id HAVING rev = 0 OR max(rev)"
            }
            QueryMode::Commited => {
                "SELECT * FROM documents WHERE id = ?1 AND rev > 0 GROUP BY id HAVING max(rev)"
            }
        }
    })?;

    let mut rows = stmt.query_and_then(params![id], utils::extract_document)?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn get_attachment(conn: &Connection, id: &Id, mode: QueryMode) -> Result<Option<Attachment>> {
    let mut stmt = conn.prepare_cached({
        match mode {
            QueryMode::All => "SELECT * FROM attachments WHERE id = ?1",
            QueryMode::Commited => "SELECT * FROM attachments WHERE id = ?1 AND rev > 0",
        }
    })?;

    let mut rows = stmt.query_and_then(params![id], utils::extract_attachment)?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn put_document(conn: &Connection, document: &Document) -> Result<()> {
    let mut stmt = conn.prepare_cached(
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

pub fn put_attachment(conn: &Connection, attachment: &Attachment) -> Result<()> {
    let mut stmt = conn.prepare_cached(
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

pub fn delete_staged_documents(conn: &Connection) -> Result<()> {
    conn.execute("DELETE * FROM documents WHERE rev = 0", NO_PARAMS)?;

    Ok(())
}

pub fn delete_staged_attachments(conn: &Connection) -> Result<()> {
    conn.execute("DELETE * FROM attachments WHERE rev = 0", NO_PARAMS)?;

    Ok(())
}
