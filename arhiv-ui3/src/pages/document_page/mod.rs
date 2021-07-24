use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{schema::Collection, Arhiv, Condition, Filter};
use rs_utils::{
    server::{respond_not_found, RequestQueryExt, ServerResponse},
    QueryBuilder,
};

use crate::{
    components::{Breadcrumb, Catalog, Ref, Toolbar},
    ui_config::UIConfig,
    utils::ArhivPageExt,
};
use fields::prepare_fields;

mod fields;

pub async fn document_page(req: Request<Body>) -> ServerResponse {
    let id: &str = req.param("id").unwrap();
    let arhiv: &Arhiv = req.data().unwrap();

    let document = {
        if let Some(document) = arhiv.get_document(&id.into())? {
            document
        } else {
            return respond_not_found();
        }
    };

    let pattern = req.get_query_param("pattern").unwrap_or("".to_string());

    let data_description = arhiv.schema.get_data_description(&document.document_type)?;
    let fields = prepare_fields(&document, arhiv, data_description)?;

    let mut children_catalog = None;

    if let Collection::Type {
        document_type: item_type,
        field,
    } = data_description.collection_of
    {
        let catalog = Catalog::new(item_type, pattern)
            .with_matcher(Condition::Field {
                field: field.to_string(),
                pattern: document.id.to_string(),
                not: false,
            })
            .with_document_url_query(
                QueryBuilder::new()
                    .add_param("parent_collection", &document.id)
                    .build(),
            )
            .with_new_document_query(
                QueryBuilder::new()
                    .add_param(field, &document.id)
                    .add_param("parent_collection", &document.id)
                    .build(),
            )
            .render(
                arhiv,
                UIConfig::get_child_config(&document.document_type, item_type).catalog,
            )?;

        children_catalog = Some(catalog);
    };

    let mut toolbar = Toolbar::new(req.get_query_param("parent_collection"))
        .with_breadcrumb(Breadcrumb::Collection(document.document_type.to_string()))
        .with_breadcrumb(Breadcrumb::Document(&document))
        .on_close_document(&document);

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
                })
            }

            filter
        })?
        .items
        .into_iter()
        .map(|document| Ref::from_document(document).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    arhiv.render_page(
        "pages/document_page.html.tera",
        json!({
            "toolbar": toolbar,
            "fields": fields,
            "refs": refs,
            "backrefs": backrefs,
            "document": document,
            "is_internal_type": data_description.is_internal,
            "children_catalog": children_catalog,
        }),
    )
}
