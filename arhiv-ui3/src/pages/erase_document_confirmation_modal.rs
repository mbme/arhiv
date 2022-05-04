use anyhow::{anyhow, ensure, Context, Result};
use hyper::StatusCode;
use serde_json::json;

use arhiv_core::entities::Id;

use crate::{
    app::{App, AppResponse},
    template_fn,
    urls::{document_url, parent_collection_url},
    utils::Fields,
};

template_fn!(
    render_template,
    "./erase_document_confirmation_modal.html.tera"
);

impl App {
    pub fn erase_document_confirmation_modal(
        &self,
        id: &Id,
        parent_collection: &Option<Id>,
    ) -> Result<AppResponse> {
        let document = self
            .arhiv
            .get_document(id)?
            .ok_or_else(|| anyhow!("document not found"))?;

        if document.is_erased() {
            let location = document_url(&document.id, &None);
            return Ok(AppResponse::MovedPermanently { location });
        }

        let document_title = self.arhiv.get_schema().get_title(&document)?;

        let content = render_template(json!({
            "id": document.id,
            "document_type": document.document_type,
            "title": document_title,
            "confirmation_text": get_confirmation_text(&document.document_type),
            "success_url":  parent_collection_url(&document.document_type, parent_collection),
        }))?;

        Ok(AppResponse::fragment(content))
    }

    pub fn erase_document_confirmation_modal_handler(
        &self,
        fields: &Fields,
    ) -> Result<AppResponse> {
        let id: Id = fields
            .get("id")
            .context("field 'id' must be present")?
            .into();

        let confirmation_text = fields
            .get("confirmation_text")
            .map(String::as_str)
            .unwrap_or_default();

        let document = self.arhiv.must_get_document(&id)?;

        ensure!(
            confirmation_text == get_confirmation_text(&document.document_type),
            "confirmation text is wrong"
        );

        let tx = self.arhiv.get_tx()?;
        tx.erase_document(&id)?;
        tx.commit()?;

        Ok(AppResponse::Status {
            status: StatusCode::NO_CONTENT,
        })
    }
}

fn get_confirmation_text(document_type: &str) -> String {
    format!("erase {}", document_type)
}
