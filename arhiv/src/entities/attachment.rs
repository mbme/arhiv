use anyhow::*;
use serde::{Deserialize, Serialize};

use super::Document;

pub const ATTACHMENT_TYPE: &'static str = "attachment";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub filename: String,
    pub hash: String,
}

pub struct Attachment(pub Document);

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

    pub fn new(data: AttachmentInfo) -> Result<Self> {
        let document = Document::new(ATTACHMENT_TYPE.to_string(), serde_json::to_value(data)?);

        Ok(Attachment(document))
    }

    pub fn get_data(&self) -> AttachmentInfo {
        serde_json::from_value(self.0.data.clone()).expect("must be able to deserialize")
    }

    pub fn set_data(&mut self, data: AttachmentInfo) -> Result<()> {
        self.0.data = serde_json::to_value(data)?;

        Ok(())
    }
}
