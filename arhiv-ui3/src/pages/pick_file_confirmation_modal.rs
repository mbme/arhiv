use std::fs;

use anyhow::{anyhow, Result};
use serde_json::json;

use arhiv_core::definitions::Attachment;
use rs_utils::{ensure_file_exists, http_server::Url};

use crate::{
    app::{App, AppResponse},
    components::Ref,
    template_fn,
    urls::pick_file_confirmation_handler_url,
    utils::Fields,
};

template_fn!(
    render_confirmation_modal,
    "./pick_file_confirmation_modal.html.tera"
);

template_fn!(
    render_confirmation_result,
    "./pick_file_confirmation_result.html.tera"
);

impl App {
    pub fn pick_file_confirmation_modal(url: &Url) -> Result<AppResponse> {
        let file_path = url
            .get_query_param("file")
            .ok_or_else(|| anyhow!("file query param must be present"))?;

        ensure_file_exists(file_path)?;

        let metadata = fs::metadata(file_path)?;
        let size = metadata.len();

        let content = render_confirmation_modal(json!({
            "handler_url": pick_file_confirmation_handler_url(),
            "file_path": file_path,
            "size": size,
        }))?;

        Ok(AppResponse::fragment(content))
    }

    pub async fn pick_file_confirmation_modal_handler(
        &self,
        fields: Fields,
    ) -> Result<AppResponse> {
        let file_path = fields
            .get("file_path")
            .ok_or_else(|| anyhow!("file_path field must be present"))?;

        let mut tx = self.arhiv.get_tx()?;
        let attachment = Attachment::create_and_stage(file_path, false, &mut tx)?;
        tx.commit()?;

        let id = attachment.id.to_string();

        let attachment_ref = Ref::from_document(attachment.into()).render(&self.arhiv)?;

        let content = render_confirmation_result(json!({
            "id": id,
            "file_path": file_path,
            "attachment_ref": attachment_ref,
        }))?;

        Ok(AppResponse::fragment(content))
    }
}
