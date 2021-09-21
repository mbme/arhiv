use chrono::{DateTime, Local};
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Catalog, Toolbar},
    pages::base::render_page,
    template_fn,
};
use arhiv_core::{entities::*, Arhiv};
use rs_utils::server::{RequestQueryExt, ServerResponse};

template_fn!(render_template, "./catalog_page.html.tera");

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

pub async fn catalog_page(req: Request<Body>) -> ServerResponse {
    let document_type: &String = req.param("document_type").unwrap();
    let arhiv: &Arhiv = req.data().unwrap();

    let pattern = req.get_query_param("pattern").unwrap_or_default();
    let page: u8 = req
        .get_query_param("page")
        .and_then(|page| page.parse().ok())
        .unwrap_or_default();

    let catalog = Catalog::new(document_type, pattern)
        .on_page(page)
        .render(arhiv)?;

    let mut toolbar = Toolbar::new(None)
        .with_breadcrumb(Breadcrumb::String(format!("{}s", document_type)))
        .on_close("/");

    let data_description = arhiv.get_schema().get_data_description(document_type)?;
    if !data_description.is_internal {
        toolbar = toolbar.with_new_document(document_type);
    }

    let toolbar = toolbar.render(arhiv)?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "document_type": document_type,
        "catalog": catalog,
    }))?;

    render_page(content, arhiv)
}
