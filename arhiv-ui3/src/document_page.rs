use anyhow::*;
use rocket::State;
use rocket_contrib::templates::Template;
use serde::Serialize;
use serde_json::json;

use arhiv::{
    markup::{MarkupRenderer, MarkupStr},
    schema::{FieldType, SCHEMA},
    Arhiv,
};

use crate::utils::TemplateContext;

#[derive(Serialize)]
struct Field {
    name: &'static str,
    value: String,
    safe: bool,
}

#[get("/documents/<id>")]
pub fn document_page(
    id: String,
    arhiv: State<Arhiv>,
    context: State<TemplateContext>,
) -> Result<Option<Template>> {
    let document = {
        if let Some(document) = arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    let renderer = MarkupRenderer::new(&arhiv, &context.markup_render_options);

    let fields = SCHEMA
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
                        "<a href=\"{0}/{1}\">{1}</a>",
                        &context.markup_render_options.document_path, value
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
        .collect::<Result<Vec<_>>>()?;

    Ok(Some(Template::render(
        "document_page",
        json!({
            "fields": fields,
            "document": document,
        }),
    )))
}
