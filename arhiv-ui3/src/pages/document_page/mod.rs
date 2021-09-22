use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{entities::Id, schema::Collection, Arhiv, Condition, Filter};
use rs_utils::server::{respond_not_found, RequestQueryExt, ServerResponse};

use crate::{
    components::{Breadcrumb, Catalog, Ref, Toolbar},
    pages::base::render_page,
    template_fn,
};
use fields::prepare_fields;

mod fields;

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

    let data_description = arhiv
        .get_schema()
        .get_data_description(&document.document_type)?;
    let fields = prepare_fields(&document, arhiv, data_description)?;

    let mut children_catalog = None;

    let mut toolbar = Toolbar::new(collection_id.clone())
        .with_breadcrumb(Breadcrumb::Collection(document.document_type.to_string()))
        .with_breadcrumb(Breadcrumb::Document(&document))
        .on_close_document(&document);

    if let Collection::Type {
        document_type: item_type,
        field,
    } = data_description.collection_of
    {
        let pattern = req.get_query_param("pattern").unwrap_or_default();

        let catalog = Catalog::new()
            .search(pattern)
            .show_search(true)
            .with_type(item_type)
            .with_matcher(Condition::Field {
                field: field.to_string(),
                pattern: document.id.to_string(),
                not: false,
            })
            .in_collection(Some(document.id.clone()))
            .render(arhiv)?;

        children_catalog = Some(catalog);

        toolbar = toolbar.with_new_collection_item(item_type, field, &document.id);
    };

    if !data_description.is_internal {
        toolbar = toolbar.with_edit(&document);
    }

    let toolbar = toolbar.render(arhiv)?;

    let refs = document
        .refs
        .iter()
        .map(|id| Ref::from_id(id).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    let backrefs = arhiv
        .list_documents({
            let mut filter = Filter::backrefs(&document.id);

            // ignore collection children
            if let Collection::Type {
                document_type,
                field,
            } = data_description.collection_of
            {
                filter = filter.with_matcher(Condition::NotCollectionChild {
                    child_document_type: document_type.to_string(),
                    child_collection_field: field.to_string(),
                    collection_id: document.id.clone(),
                });
            }

            filter
        })?
        .items
        .into_iter()
        .map(|document| Ref::from_document(document).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    let content = render_template(json!({
        "toolbar": toolbar,
        "fields": fields,
        "refs": refs,
        "backrefs": backrefs,
        "document": document,
        "is_internal_type": data_description.is_internal,
        "children_catalog": children_catalog,
        "collection_id": collection_id,
    }))?;

    render_page(content, arhiv)
}
