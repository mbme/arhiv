use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{schema::Collection, Arhiv, Filter, Matcher};
use rs_utils::{
    query_builder,
    server::{respond_not_found, RequestQueryExt, ServerResponse},
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

    if let Collection::Type(item_type) = data_description.collection_of {
        let catalog = Catalog::new(item_type, pattern)
            .with_matcher(Matcher::Field {
                selector: format!("$.{}", &document.document_type),
                pattern: document.id.to_string(),
                not: false,
            })
            .with_new_document_query(
                query_builder()
                    .append_pair(&document.document_type, &document.id)
                    .finish(),
            )
            .render(
                arhiv,
                UIConfig::get_child_config(&document.document_type, item_type).catalog,
            )?;

        children_catalog = Some(catalog);
    };

    let collection_type = arhiv.schema.get_collection_type(&document.document_type);
    let mut toolbar = Toolbar::new()
        .with_breadcrubs(vec![
            Breadcrumb::for_document_collection(&document, collection_type)?,
            Breadcrumb::for_document(&document, false),
        ])
        .on_close_document(&document, collection_type);

    if !data_description.is_internal {
        toolbar = toolbar.with_action("Edit", format!("/documents/{}/edit", &document.id));
    }

    let toolbar = toolbar.render()?;

    let refs = document
        .refs
        .iter()
        .map(|id| Ref::from_id(id).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    let backrefs = arhiv
        .list_documents({
            let mut filter = Filter::backrefs(&document.id);

            // ignore collection children
            if data_description.is_collection() {
                filter = filter.with_matcher(Matcher::Field {
                    selector: format!("$.{}", &document.document_type),
                    pattern: document.id.to_string(),
                    not: true,
                });
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
