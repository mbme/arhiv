use anyhow::*;
use async_trait::async_trait;

use arhiv_core::{definitions::Attachment, entities::Document, Arhiv};
use rs_utils::{extract_file_name_from_url, is_image_filename, log};

use crate::utils::{confirm_if_needed, Importer};

pub struct AttachmentImporter;

#[async_trait]
impl Importer for AttachmentImporter {
    fn get_name(&self) -> &str {
        "AttachmentImporter"
    }

    fn can_import(&self, url: &str) -> bool {
        extract_file_name_from_url(url)
            .ok()
            .flatten()
            .map_or(false, is_image_filename)
    }

    async fn import(&self, url: &str, arhiv: &Arhiv, confirm: bool) -> Result<Document> {
        let filename = extract_file_name_from_url(url)?
            .ok_or_else(|| anyhow!("failed to extract file name from url"))?;

        log::info!("Going to import file {}", filename);
        confirm_if_needed(confirm)?;

        let attachment = Attachment::download(url, arhiv).await?;

        Ok(attachment.into())
    }
}
