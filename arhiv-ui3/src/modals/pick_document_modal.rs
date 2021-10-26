use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::Arhiv;
use rs_utils::server::{RequestQueryExt, ServerResponse};

use super::render_modal;
use crate::{components::Catalog, template_fn};

template_fn!(render_template, "./pick_document_modal.html.tera");

pub async fn render_pick_document_modal(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let url = req.get_url();

    let catalog = Catalog::new(url).picker_mode().render(arhiv)?;

    let content = render_template(json!({
        "catalog": catalog,
    }))?;

    render_modal("pick-document-modal", "Pick document", &content, false)
}
