use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Toolbar},
    pages::base::render_page,
    template_fn,
    urls::new_document_url,
};
use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

template_fn!(render_template, "./new_document_variants_page.html.tera");

pub async fn new_document_variants_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let toolbar = Toolbar::new()
        .with_breadcrumb(Breadcrumb::string("new document"))
        .on_close("/")
        .render()?;

    let document_types = arhiv
        .get_schema()
        .get_document_types(true)
        .into_iter()
        .map(|document_type| (document_type, new_document_url(document_type, &None)))
        .collect::<Vec<_>>();

    let content = render_template(json!({
        "toolbar": toolbar, //
        "document_types": document_types,
    }))?;

    render_page(content, arhiv)
}
