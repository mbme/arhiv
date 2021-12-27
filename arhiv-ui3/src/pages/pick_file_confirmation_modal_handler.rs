use anyhow::{anyhow, Result};
use serde_json::json;

use arhiv_core::definitions::Attachment;

use crate::{
    app::{App, AppResponse},
    components::Ref,
    template_fn,
    utils::Fields,
};

template_fn!(
    render_template,
    "./pick_file_confirmation_modal_handler.html.tera"
);

impl App {
    pub async fn pick_file_confirmation_modal_handler(
        &self,
        fields: Fields,
    ) -> Result<AppResponse> {
        let file_path = fields
            .get("file_path")
            .ok_or_else(|| anyhow!("file_path field must be present"))?;

        let attachment = Attachment::create(file_path, false, &self.arhiv)?;
        let id = attachment.id.to_string();

        let attachment_ref = Ref::from_document(attachment.into()).render(&self.arhiv)?;

        let content = render_template(json!({
            "id": id,
            "file_path": file_path,
            "attachment_ref": attachment_ref,
        }))?;

        Ok(AppResponse::fragment(content))
    }
}
