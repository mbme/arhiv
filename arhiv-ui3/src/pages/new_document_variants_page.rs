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

template_fn!(render_template, "./new_document_variants_page.html.tera");

pub async fn new_document_variants_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let toolbar = Toolbar::new(None)
        .with_breadcrumb(Breadcrumb::String("new document".to_string()))
        .on_close("/")
        .render(arhiv)?;

    let document_types = arhiv
        .get_schema()
        .modules
        .iter()
        .filter(|module| !module.is_internal)
        .map(|module| module.document_type)
        .collect::<Vec<_>>();

    let content = render_template(json!({
        "toolbar": toolbar, //
        "document_types": document_types,
    }))?;

    render_page(content, arhiv)
}
