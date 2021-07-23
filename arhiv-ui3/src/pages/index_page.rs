use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Toolbar},
    utils::ArhivPageExt,
};
use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

pub async fn index_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let status = arhiv.get_status()?;

    let toolbar = Toolbar::new(None)
        .with_breadcrumb(Breadcrumb::String("index".to_string()))
        .render(arhiv)?;

    let document_types = arhiv
        .schema
        .modules
        .iter()
        .map(|module| module.document_type)
        .collect::<Vec<_>>();

    arhiv.render_page(
        "pages/index_page.html.tera",
        json!({
            "toolbar": toolbar,
            "status": status.to_string(),
            "document_types": document_types,
        }),
    )
}
