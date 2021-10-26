use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{
    definitions::PROJECT_TYPE,
    entities::{Document, Id},
    schema::Collection,
    Arhiv, Filter,
};
use rs_utils::server::{respond_not_found, RequestQueryExt, ServerResponse};

use crate::{
    components::{Action, Breadcrumb, Ref, Toolbar},
    pages::base::render_page,
    template_fn,
    urls::parent_collection_url,
};

use self::document_view::render_document_view;
use self::project_view::render_project_view;

mod document_view;
mod project_view;

template_fn!(render_template, "./document_page.html.tera");

pub async fn document_page(req: Request<Body>) -> ServerResponse {
    let id: Id = req.param("id").unwrap().into();
    let collection_id: Option<Id> = req
        .param("collection_id")
        .map(|collection_id| collection_id.into());
    let url = req.get_url();

    let arhiv: &Arhiv = req.data().unwrap();

    let document = {
        if let Some(document) = arhiv.get_document(&id)? {
            document
        } else {
            return respond_not_found();
        }
    };

    let schema = arhiv.get_schema();

    let toolbar = render_document_page_toolbar(&document, &collection_id, arhiv)?;

    let content = if document.document_type == PROJECT_TYPE {
        render_project_view(&document, arhiv, url)?
    } else {
        render_document_view(&document, arhiv, url)?
    };

    let refs = document
        .extract_refs(schema)?
        .documents
        .iter()
        .map(|id| Ref::from_id(id).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    let backrefs = arhiv
        .list_documents(Filter::backrefs(&document.id))?
        .items
        .into_iter()
        .map(|document| Ref::from_document(document).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "content": content,
        "refs": refs,
        "backrefs": backrefs,
        "document": document,
        "is_tombstone": document.is_tombstone(),
        "collection_id": collection_id,
    }))?;

    render_page(content, arhiv)
}

fn render_document_page_toolbar(
    document: &Document,
    collection_id: &Option<Id>,
    arhiv: &Arhiv,
) -> Result<String> {
    let mut toolbar = Toolbar::new()
        .with_breadcrumb(Breadcrumb::for_collection(document, collection_id, arhiv)?)
        .with_breadcrumb(Breadcrumb::for_document(document))
        .on_close(parent_collection_url(
            &document.document_type,
            collection_id,
        ));

    let schema = arhiv.get_schema();
    let data_description = schema.get_data_description(&document.document_type)?;

    if let Collection::Type {
        document_type: item_type,
        field,
    } = data_description.collection_of
    {
        toolbar = toolbar.with_action(Action::new_collection_item(item_type, field, &document.id));
    };

    if data_description.is_editable() {
        toolbar = toolbar.with_action(Action::edit(document, collection_id));
    }

    toolbar.render()
}
