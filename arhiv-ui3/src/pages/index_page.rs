use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Toolbar},
    pages::base::render_page,
    template_fn,
};
use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

template_fn!(render_template, "./index_page.html.tera");

pub async fn index_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let status = arhiv.get_status()?;

    let document_types = arhiv
        .get_schema()
        .modules
        .iter()
        .map(|module| module.document_type)
        .collect::<Vec<_>>();

    let toolbar = Toolbar::new(None)
        .with_breadcrumb(Breadcrumb::String("index".to_string()))
        .render(arhiv)?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "status": status.to_string(),
        "document_types": document_types,
    }))?;

    render_page(content, arhiv)
}
