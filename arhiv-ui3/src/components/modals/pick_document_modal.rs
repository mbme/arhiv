use anyhow::*;
use serde_json::json;

use arhiv_core::Arhiv;

use crate::{
    components::{modals::modal::render_modal, Catalog},
    template_fn,
};

template_fn!(render_template, "./pick_document_modal.html.tera");

pub fn render_pick_document_modal(arhiv: &Arhiv) -> Result<String> {
    let catalog = Catalog::new().show_search(None).render(arhiv)?;

    let content = render_template(json!({
        "catalog": catalog,
    }))?;

    render_modal("pick-document-modal", "Pick document", &content, false)
}
