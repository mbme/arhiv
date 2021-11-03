use chrono::{DateTime, Local};
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use arhiv_core::{entities::*, Arhiv};
use rs_utils::server::{RequestQueryExt, ServerResponse};

use crate::{
    components::{Action, Breadcrumb, Catalog, Toolbar},
    pages::base::render_page,
    template_fn,
};

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

    let url = req.get_url();

    let catalog = Catalog::new(url).with_type(document_type).render(arhiv)?;

    let mut toolbar = Toolbar::new()
        .with_breadcrumb(Breadcrumb::string(format!("{}s", document_type)))
        .on_close("/");

    if !arhiv.get_schema().is_internal_type(document_type) {
        toolbar = toolbar.with_action(Action::new_document(document_type, &None));
    }

    let toolbar = toolbar.render()?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "catalog": catalog,
    }))?;

    render_page(content, arhiv)
}
