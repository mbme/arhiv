use anyhow::Result;
use serde_json::json;

use arhiv_core::Filter;

use crate::{
    app::{App, AppResponse},
    components::{Breadcrumb, Ref, Toolbar},
    template_fn,
};

template_fn!(render_template, "./erased_documents_list_page.html.tera");

impl App {
    pub fn erased_documents_list_page(&self) -> Result<AppResponse> {
        let erased_documents = self
            .arhiv
            .list_documents(Filter::all_erased_documents())?
            .items
            .into_iter()
            .map(|document| Ref::from_document(document).render(&self.arhiv))
            .collect::<Result<Vec<_>>>()?;

        let toolbar = Toolbar::new()
            .with_breadcrumb(Breadcrumb::string("Erased documents"))
            .render()?;

        let content = render_template(json!({
            "toolbar": toolbar,
            "erased_documents": erased_documents,
        }))?;

        Ok(AppResponse::page("Index".to_string(), content))
    }
}
