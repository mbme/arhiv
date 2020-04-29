use crate::entities::*;
use anyhow::*;
use rusqlite::{Row, Rows};

fn extract_refs(value: String) -> serde_json::Result<Vec<Id>> {
    serde_json::from_str::<Vec<Id>>(&value)
}

pub fn serialize_refs(refs: &Vec<Id>) -> serde_json::Result<String> {
    serde_json::to_string(&refs)
}

pub fn extract_document(row: &Row) -> Result<Document> {
    Ok(Document {
        id: row.get("id")?,
        rev: row.get("rev")?,
        document_type: row.get("type")?,
        archived: row.get("archived")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        refs: extract_refs(row.get("refs")?)?,
        attachment_refs: extract_refs(row.get("attachment_refs")?)?,
        data: row.get("data")?,
    })
}

pub fn extract_attachment(row: &Row) -> Result<Attachment> {
    Ok(Attachment {
        id: row.get("id")?,
        rev: row.get("rev")?,
        created_at: row.get("created_at")?,
        filename: row.get("filename")?,
    })
}

pub fn extract_documents(rows: Rows) -> Result<Vec<Document>> {
    let mut documents = Vec::new();

    for row in rows.and_then(extract_document) {
        documents.push(row?);
    }

    Ok(documents)
}

pub fn extract_attachments(rows: Rows) -> Result<Vec<Attachment>> {
    let mut attachments = Vec::new();

    for row in rows.and_then(extract_attachment) {
        attachments.push(row?);
    }

    Ok(attachments)
}
