use anyhow::*;
use arhiv_core::entities::Document;
use async_trait::async_trait;

use rs_utils::{is_image_filename, log};

use crate::utils::{confirm_if_needed, download_file, extract_file_name_from_url, Importer};

pub struct AttachmentImporter;

#[async_trait]
impl Importer for AttachmentImporter {
    fn get_name(&self) -> &str {
        "AttachmentImporter"
    }

    fn can_import(&self, url: &str) -> bool {
        if let Some(file_name) = extract_file_name_from_url(url).ok().flatten() {
            is_image_filename(file_name)
        } else {
            false
        }
    }

    async fn import(
        &self,
        url: &str,
        arhiv: &arhiv_core::Arhiv,
        confirm: bool,
    ) -> Result<Document> {
        let filename = extract_file_name_from_url(url)?
            .ok_or(anyhow!("failed to extract file name from url"))?;

        log::info!("Going to import file {}", filename);
        confirm_if_needed(confirm)?;

        let file = download_file(url).await?;
        let attachment = arhiv.add_attachment(&file, true)?;

        Ok(attachment.into())
    }
}
