use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use arhiv::{
    entities::Document,
    markup::MarkupStr,
    schema::{DataDescription, FieldType, SCHEMA},
    Filter, Matcher, OrderBy,
};

use crate::{
    app_context::AppContext,
    components::Catalog,
    http_utils::{get_query_params, not_found, AppResponse},
};

#[derive(Serialize)]
struct Field {
    name: &'static str,
    value: String,
    safe: bool,
}

pub async fn document_page(req: Request<Body>) -> AppResponse {
    let id: &str = req.param("id").unwrap();
    let context: &AppContext = req.data().unwrap();

    let document = {
        if let Some(document) = context.arhiv.get_document(&id.into())? {
            document
        } else {
            return not_found();
        }
    };

    let mut query_params = get_query_params(req.uri());
    let pattern = query_params.remove("pattern").unwrap_or("".to_string());

    let data_description = SCHEMA.get_data_description_by_type(&document.document_type)?;
    let fields = prepare_fields(&document, &context, data_description)?;

    let refs = document
        .refs
        .iter()
        .map(|value| format!("<a href=\"/documents/{0}\">{0}</a>", value))
        .collect::<Vec<_>>();

    let children_catalog = if let Some(ref collection) = data_description.collection_of {
        let mut filter = Filter::default();
        filter.matchers.push(Matcher::Type {
            document_type: collection.item_type.to_string(),
        });
        filter.matchers.push(Matcher::Field {
            selector: format!("$.{}", document.document_type),
            pattern: document.id.to_string(),
        });
        filter.matchers.push(Matcher::Search {
            pattern: pattern.clone(),
        });
        filter.page_size = None;
        filter.page_offset = None;
        filter.order.push(OrderBy::UpdatedAt { asc: false });

        let result = context.arhiv.list_documents(filter)?;
        let catalog = Catalog::new(result.items, pattern).render(&context)?;

        Some(catalog)
    } else {
        None
    };

    context.render_page(
        "pages/document_page.html.tera",
        json!({
            "refs": refs,
            "fields": fields,
            "document": document,
            "children_catalog": children_catalog,
            "is_tombstone": document.is_tombstone(),
        }),
    )
}

fn prepare_fields(
    document: &Document,
    context: &AppContext,
    data_description: &DataDescription,
) -> Result<Vec<Field>> {
    data_description
        .fields
        .iter()
        .map(|field| {
            let value = document.get_field_str(field.name);

            match (&field.field_type, value) {
                (FieldType::MarkupString {}, _) => {
                    let markup: MarkupStr = value.unwrap_or("").into();

                    Ok(Field {
                        name: field.name,
                        value: context.render_markup(&markup),
                        safe: true,
                    })
                }
                (FieldType::Ref(_), Some(value)) => Ok(Field {
                    name: field.name,
                    value: format!("<a href=\"/documents/{0}\">{0}</a>", value),
                    safe: true,
                }),
                _ => Ok(Field {
                    name: field.name,
                    value: value.unwrap_or("").to_string(),
                    safe: false,
                }),
            }
        })
        .collect::<Result<Vec<_>>>()
}
