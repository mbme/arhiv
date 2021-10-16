use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{entities::Id, schema::Collection, Arhiv, Filter};
use rs_utils::server::{respond_not_found, RequestQueryExt, ServerResponse};

use crate::{
    components::{Action, Breadcrumb, Catalog, DocumentDataViewer, Ref, Toolbar},
    pages::base::render_page,
    template_fn,
    urls::parent_collection_url,
};

template_fn!(render_template, "./document_page.html.tera");

pub async fn document_page(req: Request<Body>) -> ServerResponse {
    let id: Id = req.param("id").unwrap().into();
    let collection_id: Option<Id> = req
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = req.data().unwrap();

    let document = {
        if let Some(document) = arhiv.get_document(&id)? {
            document
        } else {
            return respond_not_found();
        }
    };

    let schema = arhiv.get_schema();
    let data_description = schema.get_data_description(&document.document_type)?;

    let mut children_catalog = None;

    let mut toolbar = Toolbar::new()
        .with_breadcrumb(Breadcrumb::for_collection(
            &document,
            &collection_id,
            arhiv,
        )?)
        .with_breadcrumb(Breadcrumb::for_document(&document))
        .on_close(parent_collection_url(
            &document.document_type,
            &collection_id,
        ));

    if let Collection::Type {
        document_type: item_type,
        field: _field,
    } = data_description.collection_of
    {
        let pattern = req.get_query_param("pattern").unwrap_or_default();

        let catalog = Catalog::new()
            .search(pattern)
            .show_search(Some("pattern"))
            .with_type(item_type)
            .in_collection(document.id.clone())
            .render(arhiv)?;

        children_catalog = Some(catalog);

        toolbar = toolbar.with_action(Action::new_collection_item(&document, arhiv)?);
    };

    if !data_description.is_internal {
        toolbar = toolbar.with_action(Action::edit(&document, &collection_id));
    }

    let toolbar = toolbar.render()?;

    let viewer = DocumentDataViewer::new(&document).render(arhiv)?;

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
        "viewer": viewer,
        "refs": refs,
        "backrefs": backrefs,
        "document": document,
        "is_internal_type": data_description.is_internal,
        "children_catalog": children_catalog,
        "collection_id": collection_id,
    }))?;

    render_page(content, arhiv)
}
