use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use crate::{
    components::{Breadcrumbs, Catalog},
    markup::ArhivMarkupExt,
    ui_config::CatalogConfig,
    utils::render_page,
};
use arhiv_core::{
    entities::Document,
    markup::MarkupStr,
    schema::{DataDescription, FieldType, SCHEMA},
    Arhiv, Matcher,
};
use rs_utils::{
    query_builder,
    server::{respond_not_found, RequestQueryExt, ServerResponse},
};

#[derive(Serialize)]
struct Field {
    name: &'static str,
    value: String,
    is_safe: bool,
    is_title: bool,
}

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

    let data_description = SCHEMA.get_data_description_by_type(&document.document_type)?;
    let fields = prepare_fields(&document, arhiv, data_description)?;

    let mut children_catalog = None;

    if let Some(ref collection) = data_description.collection_of {
        let catalog = Catalog::new(collection.item_type, pattern)
            .with_matcher(Matcher::Field {
                selector: format!("$.{}", &document.document_type),
                pattern: document.id.to_string(),
            })
            .with_new_document_query(
                query_builder()
                    .append_pair(&document.document_type, &document.id)
                    .finish(),
            )
            .render(
                arhiv,
                CatalogConfig::get_child_config(&document.document_type, &collection.item_type),
            )?;

        children_catalog = Some(catalog);
    };

    let breadcrumbs = Breadcrumbs::Document(&document).render()?;

    render_page(
        "pages/document_page.html.tera",
        json!({
            "breadcrumbs": breadcrumbs,
            "fields": fields,
            "document": document,
            "is_internal_type": data_description.is_internal,
            "children_catalog": children_catalog,
        }),
    )
}

fn prepare_fields(
    document: &Document,
    arhiv: &Arhiv,
    data_description: &DataDescription,
) -> Result<Vec<Field>> {
    let title_field = SCHEMA.pick_title_field(&document.document_type)?;

    data_description
        .fields
        .iter()
        .map(|field| {
            let value = document.get_field_str(field.name);
            let is_title = field.name == title_field.name;

            match (&field.field_type, value) {
                (FieldType::MarkupString {}, _) => {
                    let markup: MarkupStr = value.unwrap_or("").into();

                    Ok(Field {
                        name: field.name,
                        value: arhiv.render_markup(&markup),
                        is_safe: true,
                        is_title,
                    })
                }
                (FieldType::Ref(_), Some(value)) => Ok(Field {
                    name: field.name,
                    value: format!("<a href=\"/documents/{0}\">{0}</a>", value),
                    is_safe: true,
                    is_title,
                }),
                _ => Ok(Field {
                    name: field.name,
                    value: value.unwrap_or("").to_string(),
                    is_safe: true,
                    is_title,
                }),
            }
        })
        .collect::<Result<Vec<_>>>()
}
