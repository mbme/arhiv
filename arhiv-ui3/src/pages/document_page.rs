use anyhow::*;
use rocket::State;
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use arhiv::{
    entities::Document,
    markup::MarkupStr,
    schema::{FieldType, SCHEMA},
};

use crate::utils::AppContext;

#[derive(Serialize)]
struct Field {
    name: &'static str,
    value: String,
    safe: bool,
}

#[get("/documents/<id>")]
pub fn document_page(id: String, context: State<AppContext>) -> Result<Option<Template>> {
    let document = {
        if let Some(document) = context.arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    let fields = prepare_fields(&document, &context)?;

    Ok(Some(Template::render(
        "pages/document_page",
        json!({
            "fields": fields,
            "document": document,
        }),
    )))
}

fn prepare_fields(document: &Document, context: &AppContext) -> Result<Vec<Field>> {
    let renderer = context.get_renderer();

    SCHEMA
        .get_data_description_by_type(&document.document_type)?
        .fields
        .iter()
        .map(|field| {
            let value = document
                .data
                .get(field.name)
                .map(|value| value.as_str())
                .flatten();

            match (&field.field_type, value) {
                (FieldType::MarkupString {}, _) => {
                    let markup: MarkupStr = value.unwrap_or("").into();

                    Ok(Field {
                        name: field.name,
                        value: renderer.to_html(&markup),
                        safe: true,
                    })
                }
                (FieldType::Ref(_), Some(value)) => Ok(Field {
                    name: field.name,
                    value: format!(
                        "<a href=\"{0}\">{1}</a>",
                        context.get_document_url(value),
                        value
                    ),
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
