use anyhow::*;
use arhiv::{
    entities::Document,
    schema::{DataDescription, FieldType, SCHEMA},
};
use rocket::State;
use serde::Serialize;
use serde_json::json;

use crate::app_context::{AppContext, TemplatePage};

#[get("/documents/<id>/edit")]
pub fn document_editor_page(
    id: String,
    context: State<AppContext>,
) -> Result<Option<TemplatePage>> {
    let document = {
        if let Some(document) = context.arhiv.get_document(&id.into())? {
            document
        } else {
            return Ok(None);
        }
    };

    let data_description = SCHEMA.get_data_description_by_type(&document.document_type)?;
    let fields = prepare_fields(&document, data_description)?;

    Ok(Some(context.render_page(
        "pages/document_editor_page.html.tera",
        json!({
            "document": document, //
            "fields": fields,
        }),
    )?))
}

#[derive(Serialize)]
struct FormField {
    name: &'static str,
    label: String,
    field_type: FieldType,
    optional: bool,
    value: String,
}

fn prepare_fields(
    document: &Document,
    data_description: &DataDescription,
) -> Result<Vec<FormField>> {
    data_description
        .fields
        .iter()
        .map(|field| {
            let value = document.get_field_str(field.name).unwrap_or("");

            let mut field = FormField {
                name: field.name,
                label: field.name.to_string(),
                field_type: field.field_type.clone(),
                optional: field.optional,
                value: value.to_string(),
            };

            match &field.field_type {
                FieldType::Ref(to) => {
                    field.label = format!("{} (Ref to {})", field.name, to);
                }
                _ => {}
            }

            Ok(field)
        })
        .collect::<Result<Vec<_>>>()
}
