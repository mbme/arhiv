use std::{convert::TryInto, ops::Deref};

use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Document;
use rs_utils::{get_file_hash_sha256, get_file_name, is_image_filename};

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

        let document = Document::new_with_data(
            ATTACHMENT_TYPE.to_string(),
            json!({
                "filename": filename,
                "sha256": sha256,
            })
            .try_into()?,
        );

        Ok(Attachment(document))
    }

    pub fn get_hash(&self) -> &str {
        self.data.get_mandatory_str("sha256")
    }

    pub fn get_filename(&self) -> &str {
        self.data.get_mandatory_str("filename")
    }

    pub fn is_image(&self) -> bool {
        let filename = self.get_filename().to_lowercase();

        is_image_filename(filename)
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

impl TryInto<Attachment> for Document {
    type Error = Error;

    fn try_into(self) -> Result<Attachment, Self::Error> {
        Attachment::from(self)
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
