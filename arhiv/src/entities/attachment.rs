use std::ops::Deref;

use anyhow::*;
use serde::{Deserialize, Serialize};

use super::{Document, Hash};

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

    pub fn new(filename: String, hash: Hash) -> Self {
        let document = Document::new(
            ATTACHMENT_TYPE.to_string(),
            AttachmentInfo { filename, hash }.into(),
        );

        Attachment(document)
    }

    fn get_data(&self) -> AttachmentInfo {
        serde_json::from_value(self.0.data.clone()).expect("must be able to deserialize")
    }

    pub fn get_hash(&self) -> Hash {
        self.get_data().hash
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
#[serde(rename_all = "camelCase")]
struct AttachmentInfo {
    pub filename: String,
    pub hash: Hash,
}

impl Into<serde_json::Value> for AttachmentInfo {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(self).expect("must be able to serialize")
    }
}
