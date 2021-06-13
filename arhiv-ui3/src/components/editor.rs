use anyhow::*;
use serde::Serialize;

use crate::templates::TEMPLATES;
use arhiv_core::{
    entities::Document,
    schema::{FieldType, SCHEMA},
};

#[derive(Serialize)]
struct FormField {
    name: &'static str,
    label: String,
    field_type: FieldType,
    optional: bool,
    value: String,
}

#[derive(Serialize)]
pub struct Editor<'d> {
    fields: Vec<FormField>,
    document: &'d Document,
}

impl<'d> Editor<'d> {
    pub fn new(document: &'d Document) -> Result<Self> {
        let data_description = SCHEMA.get_data_description_by_type(&document.document_type)?;

        let fields = data_description
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
            .collect::<Result<Vec<_>>>()?;

        Ok(Editor { fields, document })
    }

    pub fn render(self) -> Result<String> {
        TEMPLATES.render("components/editor.html.tera", self)
    }
}
