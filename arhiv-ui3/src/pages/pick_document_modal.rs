use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::Arhiv;
use rs_utils::server::{RequestQueryExt, ServerResponse};

use crate::{components::Catalog, pages::base::render_modal, template_fn};

template_fn!(render_template, "./pick_document_modal.html.tera");

pub async fn pick_document_modal(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let url = req.get_url();

    let catalog = Catalog::new(url).picker_mode().render(arhiv)?;

    let content = render_template(json!({
        "catalog": catalog,
    }))?;

    render_modal(content)
}
