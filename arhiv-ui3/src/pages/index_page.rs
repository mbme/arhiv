use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Toolbar},
    utils::render_page,
};
use arhiv_core::{schema::SCHEMA, Arhiv};
use rs_utils::server::ServerResponse;

pub async fn index_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let status = arhiv.get_status()?;

    let toolbar = Toolbar::new()
        .with_breadcrubs(vec![
            Breadcrumb::for_string("index"), //
        ])
        .render()?;

    let document_types = SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .collect::<Vec<_>>();

    render_page(
        "pages/index_page.html.tera",
        json!({
            "toolbar": toolbar,
            "status": status.to_string(),
            "document_types": document_types,
        }),
    )
}
