use anyhow::Result;
use chrono::{DateTime, Local};
use serde::Serialize;
use serde_json::json;

use arhiv_core::entities::*;
use rs_utils::server::Url;

use crate::{
    app::{App, AppResponse},
    components::{Action, Breadcrumb, Catalog, Toolbar},
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

impl App {
    pub fn catalog_page(&self, document_type: &str, url: Url) -> Result<AppResponse> {
        let catalog = Catalog::new(url)
            .with_type(document_type)
            .render(&self.arhiv)?;

        let mut toolbar = Toolbar::new()
            .with_breadcrumb(Breadcrumb::string(format!("{}s", document_type)))
            .on_close("/");

        if document_type != ERASED_DOCUMENT_TYPE {
            toolbar = toolbar.with_action(Action::new_document(document_type, &None));
        }

        let toolbar = toolbar.render()?;

        let content = render_template(json!({
            "toolbar": toolbar,
            "catalog": catalog,
        }))?;

        let title = format!("{}s catalog", document_type);

        Ok(AppResponse::page(title, content))
    }
}
