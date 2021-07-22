use anyhow::*;
use serde::Serialize;

use crate::templates::TEMPLATES;
use arhiv_core::{
    entities::Document,
    schema::{DataDescription, FieldType},
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
    pub fn new(document: &'d Document, data_description: &DataDescription) -> Result<Self> {
        let fields = data_description
            .fields
            .iter()
            .map(|field| {
                let value = match &field.field_type {
                    FieldType::Flag {} => {
                        let value = document
                            .data
                            .get(field.name)
                            .and_then(|value| value.as_bool())
                            .unwrap_or(false);

                        value.to_string()
                    }
                    _ => document.data.get_str(field.name).unwrap_or("").to_string(),
                };

                let mut field = FormField {
                    name: field.name,
                    label: field.name.to_string(),
                    field_type: field.field_type.clone(),
                    optional: field.optional,
                    value,
                };

                match &field.field_type {
                    FieldType::Ref(to) => {
                        field.label = format!("{} (Ref to {})", field.name, to);
                    }
                    FieldType::RefList(to) => {
                        field.label = format!("{} (Refs to {})", field.name, to);
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
