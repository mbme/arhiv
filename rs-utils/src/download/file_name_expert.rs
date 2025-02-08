use anyhow::Result;
use url::Url;

use crate::{
    get_extension_for_mime_string, get_file_extension, http::extract_file_name_from_url,
    infer_extension_by_file_mime_type,
};

#[derive(Debug)]
pub struct DownloadFileNameExpert<'u> {
    pub url: &'u Url,
    pub attachment_file_name: String,
    pub content_type: Option<String>,
    pub file_path: String,
}

impl DownloadFileNameExpert<'_> {
    pub fn deduce_file_name(self) -> Result<String> {
        // use Content-Disposition header if any
        if !self.attachment_file_name.is_empty() {
            return Ok(self.attachment_file_name);
        }

        // try to guess from url, if file has an extension
        let constructed_name = extract_file_name_from_url(self.url);
        if get_file_extension(&constructed_name).is_some() {
            return Ok(constructed_name);
        }

        // guess extension from the Content-Type header
        if let Some(content_type) = self.content_type {
            if let Some(extension) = get_extension_for_mime_string(&content_type) {
                return Ok(format!("{constructed_name}.{extension}"));
            }
        }

        // guess extension from the file data
        if let Some(extension) = infer_extension_by_file_mime_type(&self.file_path)? {
            return Ok(format!("{constructed_name}.{extension}"));
        }

        Ok(constructed_name)
    }
}
