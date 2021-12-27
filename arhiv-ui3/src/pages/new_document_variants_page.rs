use anyhow::Result;
use serde_json::json;

use crate::{
    app::{App, AppResponse},
    components::{Breadcrumb, Toolbar},
    template_fn,
    urls::new_document_url,
};

template_fn!(render_template, "./new_document_variants_page.html.tera");

impl App {
    pub fn new_document_variants_page(&self) -> Result<AppResponse> {
        let toolbar = Toolbar::new()
            .with_breadcrumb(Breadcrumb::string("new document"))
            .on_close("/")
            .render()?;

        let document_types = self
            .arhiv
            .get_schema()
            .get_document_types(true)
            .into_iter()
            .map(|document_type| (document_type, new_document_url(document_type, &None)))
            .collect::<Vec<_>>();

        let content = render_template(json!({
            "toolbar": toolbar, //
            "document_types": document_types,
        }))?;

        Ok(AppResponse::content(content))
    }
}
