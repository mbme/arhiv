use crate::entities::*;
use anyhow::*;
use path_manager::PathManager;
use rusqlite::{params, Connection, OpenFlags, NO_PARAMS};

mod path_manager;
mod utils;

pub struct Storage {
    path_manager: PathManager,
}

impl Storage {
    pub fn open(root_path: &str) -> Result<Storage> {
        let path_manager = PathManager::new(root_path.to_string());
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        Ok(Storage { path_manager })
    }

    pub fn create(root_path: &str) -> Result<Storage> {
        let path_manager = PathManager::new(root_path.to_string());
        path_manager.create_dirs()?;

        let conn = Connection::open(path_manager.get_db_file())?;
        conn.execute_batch(include_str!("./schema.sql"))?;

        Ok(Storage { path_manager })
    }

    pub fn get_connection(&self) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            self.path_manager.get_db_file(),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;

        Ok(conn)
    }

    pub fn get_writable_connection(&self) -> Result<Connection> {
        let conn = Connection::open_with_flags(
            self.path_manager.get_db_file(),
            OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;

        Ok(conn)
    }

    pub fn get_attachment_file_path(&self, id: &Id) -> String {
        format!("{}/{}", self.path_manager.get_data_directory(), id)
    }
}

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

pub fn get_all_documents(conn: &Connection) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare_cached(
        "SELECT * FROM documents GROUP BY id HAVING rev = 0 OR max(rev) ORDER BY rev DESC",
    )?;

    let row = stmt.query(NO_PARAMS)?;

    utils::extract_documents(row)
}

pub fn get_commited_documents(conn: &Connection) -> Result<Vec<Document>> {
    get_commited_documents_with_rev(conn, 1)
}

pub fn get_commited_documents_with_rev(
    conn: &Connection,
    min_rev: Revision,
) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare_cached(
        "SELECT * FROM documents WHERE rev >= ?1 GROUP BY id HAVING max(rev) ORDER BY rev DESC",
    )?;

    let row = stmt.query(params![min_rev])?;

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

pub enum QueryMode {
    All,
    Commited,
    Staged,
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
            QueryMode::Staged => "SELECT * FROM documents WHERE id = ?1 AND rev = 0",
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
            QueryMode::Staged => "SELECT * FROM attachments WHERE id = ?1 AND rev = 0",
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
