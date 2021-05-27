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
        "document_page",
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
        .map(|field| match field.field_type {
            FieldType::MarkupString {} => {
                let markup: MarkupStr = document.get_field_str(field.name)?.into();

                Ok(Field {
                    name: field.name,
                    value: renderer.to_html(&markup),
                    safe: true,
                })
            }
            FieldType::Ref(_) => {
                let value = document.get_field_str(field.name)?;

                Ok(Field {
                    name: field.name,
                    value: format!(
                        "<a href=\"{0}\">{1}</a>",
                        context.get_document_url(value),
                        value
                    ),
                    safe: true,
                })
            }
            _ => {
                let value = document.get_field_str(field.name)?.to_string();

                Ok(Field {
                    name: field.name,
                    value,
                    safe: false,
                })
            }
        })
        .collect::<Result<Vec<_>>>()
}
