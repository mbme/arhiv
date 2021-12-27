use anyhow::Result;
use serde_json::json;

use crate::{
    app::{App, AppResponse},
    components::{Breadcrumb, Toolbar},
    template_fn,
    urls::catalog_url,
};

template_fn!(render_template, "./index_page.html.tera");

impl App {
    pub fn index_page(&self) -> Result<AppResponse> {
        let status = self.arhiv.get_status()?;

        let document_types = self
            .arhiv
            .get_schema()
            .get_document_types(false)
            .into_iter()
            .map(|document_type| (document_type, catalog_url(document_type)))
            .collect::<Vec<_>>();

        let toolbar = Toolbar::new()
            .with_breadcrumb(Breadcrumb::string("index"))
            .render()?;

        let content = render_template(json!({
            "toolbar": toolbar,
            "status": status.to_string(),
            "document_types": document_types,
        }))?;

        Ok(AppResponse::page(content))
    }
}
