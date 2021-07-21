use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Toolbar},
    utils::ArhivPageExt,
};
use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

pub async fn new_document_variants_page(req: Request<Body>) -> ServerResponse {
    let toolbar = Toolbar::new(None)
        .with_breadcrubs(vec![
            Breadcrumb::String("new document".to_string()), //
        ])
        .on_close("/")
        .render()?;

    let arhiv: &Arhiv = req.data().unwrap();

    let document_types = arhiv
        .schema
        .modules
        .iter()
        .filter(|module| !module.is_internal)
        .map(|module| module.document_type)
        .collect::<Vec<_>>();

    arhiv.render_page(
        "pages/new_document_variants_page.html.tera",
        json!({
            "toolbar": toolbar, //
            "document_types": document_types,
        }),
    )
}
