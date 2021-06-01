use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use crate::app_context::AppContext;
use arhiv::schema::{FieldType, SCHEMA};

#[derive(Serialize)]
struct FormField {
    name: &'static str,
    label: String,
    field_type: FieldType,
    optional: bool,
    value: String,
}

#[derive(Serialize)]
pub struct Editor {
    fields: Vec<FormField>,
    cancel_url: Option<String>,
}

impl Editor {
    pub fn new(document_type: &str, data: &Value, cancel_url: Option<String>) -> Result<Self> {
        let data_description = SCHEMA.get_data_description_by_type(document_type)?;

        let fields = data_description
            .fields
            .iter()
            .map(|field| {
                let value = data
                    .get(field.name)
                    .map(|value| value.as_str())
                    .flatten()
                    .unwrap_or("");

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

        Ok(Editor { fields, cancel_url })
    }

    pub fn render(self, context: &AppContext) -> Result<String> {
        context.render_template("components/editor.html.tera", self)
    }
}
