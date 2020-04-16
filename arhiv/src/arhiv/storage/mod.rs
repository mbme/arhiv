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
}

pub fn get_rev(conn: &Connection) -> Result<Revision> {
    let rev = conn.query_row(
        "SELECT IFNULL(MAX(rev), 0) FROM (SELECT rev FROM documents UNION ALL SELECT rev FROM attachments)",
        NO_PARAMS,
        |row| row.get(0),
    )?;

    Ok(rev)
}

pub fn get_documents(conn: &Connection) -> Result<Vec<Document>> {
    // FIXME https://stackoverflow.com/a/48328243
    let mut stmt = conn.prepare_cached("SELECT * FROM documents ORDER BY rev DESC LIMIT 10")?;

    let rows = stmt.query_and_then(NO_PARAMS, utils::extract_document)?;

    let mut documents = Vec::new();

    for row in rows {
        documents.push(row?);
    }

    Ok(documents)
}

pub fn get_attachments(conn: &Connection) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare_cached("SELECT * FROM attachments ORDER BY rev DESC LIMIT 10")?;

    let rows = stmt.query_and_then(NO_PARAMS, utils::extract_attachment)?;

    let mut attachments = Vec::new();

    for row in rows {
        attachments.push(row?);
    }

    Ok(attachments)
}

pub fn put_document(conn: &Connection, document: &Document) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO documents
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
