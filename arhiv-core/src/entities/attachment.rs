use std::ops::Deref;

use anyhow::*;
use serde::{Deserialize, Serialize};

use super::Document;
use rs_utils::{get_file_hash_sha256, get_file_name};

pub const ATTACHMENT_TYPE: &'static str = "attachment";

pub struct Attachment(Document);

impl Attachment {
    pub fn is_attachment(document: &Document) -> bool {
        document.document_type == ATTACHMENT_TYPE
    }

    pub fn from(document: Document) -> Result<Self> {
        ensure!(
            Attachment::is_attachment(&document),
            "document {} must be an attachment",
            &document.id,
        );

        Ok(Attachment(document))
    }

    pub fn new(file_path: &str) -> Result<Self> {
        let sha256 = get_file_hash_sha256(file_path)?;
        let filename = get_file_name(file_path).to_string();

        let document = Document::new(
            ATTACHMENT_TYPE.to_string(),
            AttachmentInfo { filename, sha256 }.into(),
        );

        Ok(Attachment(document))
    }

    fn get_data(&self) -> AttachmentInfo {
        serde_json::from_value(self.0.data.clone()).expect("must be able to deserialize")
    }

    pub fn get_hash(&self) -> String {
        self.get_data().sha256
    }

    pub fn get_filename(&self) -> String {
        self.get_data().filename
    }
}

impl Deref for Attachment {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<Document> for Attachment {
    fn into(self) -> Document {
        self.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct AttachmentInfo {
    pub filename: String,
    pub sha256: String,
}

impl Into<serde_json::Value> for AttachmentInfo {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(self).expect("must be able to serialize")
    }
}
