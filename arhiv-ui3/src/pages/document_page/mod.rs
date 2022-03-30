use std::ops::Not;

use anyhow::{Context, Result};
use hyper::StatusCode;
use serde_json::json;

use arhiv_core::{
    definitions::PROJECT_TYPE,
    entities::{Document, Id, ERASED_DOCUMENT_TYPE},
    schema::Collection,
    Arhiv, Filter,
};
use rs_utils::http_server::Url;

use crate::{
    app::{App, AppResponse},
    components::{Action, Breadcrumb, Ref, Toolbar},
    template_fn,
    urls::{erase_document_url, index_url, parent_collection_url},
};

use self::document_view::render_document_view;
use self::project_view::render_project_view;

mod document_view;
mod project_view;

template_fn!(render_template, "./document_page.html.tera");
template_fn!(
    render_erased_document_template,
    "./erased_document_view.html.tera"
);

impl App {
    pub fn document_page(
        &self,
        id: &Id,
        collection_id: &Option<Id>,
        url: Url,
    ) -> Result<AppResponse> {
        let document = {
            if let Some(document) = self.arhiv.get_document(id)? {
                document
            } else {
                return Ok(AppResponse::status(StatusCode::NOT_FOUND));
            }
        };

        let toolbar = render_document_page_toolbar(&document, collection_id, &self.arhiv)?;

        let content = if document.document_type == PROJECT_TYPE {
            render_project_view(&document, &self.arhiv, &url)?
        } else if document.document_type == ERASED_DOCUMENT_TYPE {
            render_erased_document_template(json!({}))?
        } else {
            render_document_view(&document, &self.arhiv, url)?
        };

        let backrefs = self
            .arhiv
            .list_documents(Filter::all_backrefs(&document.id))?
            .items
            .into_iter()
            .map(|document| Ref::from_document(document).render(&self.arhiv))
            .collect::<Result<Vec<_>>>()?;

        let content = render_template(json!({
            "toolbar": toolbar,
            "content": content,
            "backrefs": backrefs,
            "erase_document_url": document.is_erased().not().then(|| erase_document_url(&document.id, collection_id)),
        }))?;

        let title = format!(
            "{} {}",
            document.document_type,
            self.arhiv.get_schema().get_title(&document)?
        );

        Ok(AppResponse::page(title, content))
    }

    pub fn document_api(&self, id: &Id) -> Result<AppResponse> {
        if let Some(document) = self.arhiv.get_document(id)? {
            let content =
                serde_json::to_string(&document).context("failed to serialize document")?;

            Ok(AppResponse::json(content))
        } else {
            Ok(AppResponse::status(StatusCode::NOT_FOUND))
        }
    }
}

fn render_document_page_toolbar(
    document: &Document,
    collection_id: &Option<Id>,
    arhiv: &Arhiv,
) -> Result<String> {
    let mut toolbar = Toolbar::new();

    if document.is_erased() {
        toolbar = toolbar
            .with_breadcrumb(Breadcrumb::string("ERASED DOCUMENT"))
            .on_close(index_url());
    } else {
        toolbar = toolbar
            .with_breadcrumb(Breadcrumb::for_collection(
                &document.document_type,
                collection_id,
                arhiv,
            )?)
            .with_breadcrumb(Breadcrumb::for_document(document))
            .on_close(parent_collection_url(
                &document.document_type,
                collection_id,
            ));
    }

    let schema = arhiv.get_schema();
    let data_description = schema.get_data_description(&document.document_type)?;

    if let Collection::Type {
        document_type: item_type,
        field: _,
    } = data_description.collection_of
    {
        toolbar = toolbar.with_action(Action::new_document(item_type, &Some(document.id.clone())));
    };

    if data_description.is_editable() {
        toolbar = toolbar.with_action(Action::edit(document, collection_id));
    }

    toolbar.render()
}
