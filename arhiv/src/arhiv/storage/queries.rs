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
) -> Result<QueryPage<Document>> {
    let mut query =
        vec!["SELECT *, COUNT(*) OVER() AS total_count FROM documents WHERE rev >= :min_rev"];
    let mut params: Vec<(&str, &dyn ToSql)> = vec![(":min_rev", &min_rev)];

    if let Some(ref document_type) = filter.document_type {
        query.push("AND type = :type");
        params.push((":type", document_type));
    }

    // local documents with rev === 0 have higher priority
    query.push("GROUP BY id ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END)");

    if let Some(ref page_size) = filter.page_size {
        query.push("LIMIT :limit");
        params.push((":limit", page_size));
    }

    if let Some(ref page_offset) = filter.page_offset {
        query.push("OFFSET :offset");
        params.push((":offset", page_offset));
    }

    let mut stmt = conn.prepare_cached(&query.join(" "))?;

    let mut rows = stmt.query_named(&params)?;

    let mut documents = Vec::new();
    let mut total = Option::None;
    while let Some(row) = rows.next()? {
        if total.is_none() {
            total = Some(row.get("total_count")?);
        }

        documents.push(utils::extract_document(row)?);
    }

    Ok(QueryPage {
        results: documents,
        total: total.unwrap_or(0),
        page_size: filter.page_size,
        page_offset: filter.page_offset,
    })
}

pub fn get_staged_documents(conn: &Connection) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM documents WHERE rev = 0")?;

    let mut rows = stmt.query(NO_PARAMS)?;

    let mut documents = Vec::new();
    while let Some(row) = rows.next()? {
        documents.push(utils::extract_document(row)?);
    }

    Ok(documents)
}

pub fn get_all_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare_cached(
        "SELECT * FROM attachments GROUP BY id ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END)",
    )?;

    let mut rows = stmt.query(NO_PARAMS)?;

    let mut attachments = Vec::new();
    while let Some(row) = rows.next()? {
        attachments.push(utils::extract_attachment(row)?);
    }

    Ok(attachments)
}

pub fn get_commited_attachments_with_rev(
    conn: &Connection,
    min_rev: Revision,
) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM attachments WHERE rev >= ?1 GROUP BY id")?;

    let mut rows = stmt.query(params![min_rev])?;

    let mut attachments = Vec::new();
    while let Some(row) = rows.next()? {
        attachments.push(utils::extract_attachment(row)?);
    }

    Ok(attachments)
}

pub fn get_commited_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    get_commited_attachments_with_rev(conn, 1)
}

pub fn get_staged_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM attachments WHERE rev = 0")?;

    let mut rows = stmt.query(NO_PARAMS)?;

    let mut attachments = Vec::new();
    while let Some(row) = rows.next()? {
        attachments.push(utils::extract_attachment(row)?);
    }

    Ok(attachments)
}

pub fn get_document(conn: &Connection, id: &Id, mode: QueryMode) -> Result<Option<Document>> {
    let mut stmt = conn.prepare_cached({
        match mode {
            QueryMode::All => {
                "SELECT * FROM documents WHERE id = ?1 ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END) LIMIT 1"
            }
            QueryMode::Commited => {
                "SELECT * FROM documents WHERE id = ?1 AND rev > 0 GROUP BY id HAVING MAX(rev)"
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
            QueryMode::All => "SELECT * FROM attachments WHERE id = ?1 ORDER BY (CASE WHEN rev = 0 THEN 1 ELSE 2 END) LIMIT 1",
            QueryMode::Commited => "SELECT * FROM attachments WHERE id = ?1 AND rev > 0 GROUP BY id HAVING MAX(rev)",
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
    let rows = conn.execute("DELETE FROM documents WHERE rev = 0", NO_PARAMS)?;

    log::debug!("deleted {} staged documents", rows);

    Ok(())
}

pub fn delete_staged_attachments(conn: &Connection) -> Result<()> {
    let rows = conn.execute("DELETE FROM attachments WHERE rev = 0", NO_PARAMS)?;

    log::debug!("deleted {} staged attachments", rows);

    Ok(())
}

pub fn get_changeset(conn: &Connection) -> Result<Changeset> {
    let changeset = Changeset {
        base_rev: get_rev(conn)?,
        documents: get_staged_documents(conn)?,
        attachments: get_staged_attachments(conn)?,
    };
    log::debug!("prepared a changeset {}", changeset);

    Ok(changeset)
}
