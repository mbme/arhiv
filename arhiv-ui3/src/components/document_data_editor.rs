use anyhow::*;
use serde::Serialize;

use arhiv_core::{
    entities::DocumentData,
    schema::{DataDescription, FieldType},
    FieldValidationErrors,
};

use crate::template_fn;

template_fn!(render_template, "./document_data_editor.html.tera");

#[derive(Serialize)]
struct FormField {
    name: &'static str,
    label: String,
    field_type: FieldType,
    mandatory: bool,
    value: String,
    editable: bool,
    errors: Vec<String>,
}

#[derive(Serialize)]
pub struct DocumentDataEditor {
    fields: Vec<FormField>,
}

impl DocumentDataEditor {
    pub fn new(data: &DocumentData, data_description: &DataDescription) -> Result<Self> {
        let fields = data_description
            .fields
            .iter()
            .map(|field| {
                let value = match &field.field_type {
                    FieldType::Flag {} => data.get_bool(field.name).unwrap_or(false).to_string(),

                    FieldType::NaturalNumber {} => data
                        .get_number(field.name)
                        .map(|value| value.to_string())
                        .unwrap_or_default(),

                    FieldType::RefList(_) => {
                        let value = data.get(field.name);

                        if let Some(value) = value {
                            let refs: Vec<String> = serde_json::from_value(value.clone())?;

                            refs.join(" ")
                        } else {
                            "".to_string()
                        }
                    }

                    _ => data.get_str(field.name).unwrap_or_default().to_string(),
                };

                let mut field = FormField {
                    name: field.name,
                    label: field.name.to_string(),
                    field_type: field.field_type.clone(),
                    mandatory: field.mandatory,
                    value,
                    editable: true,
                    errors: vec![],
                };

                match &field.field_type {
                    FieldType::Ref(to) => {
                        field.label = format!("{} (Ref to {})", field.name, to);
                    }
                    FieldType::RefList(to) => {
                        field.label = format!("{} (Refs to {})", field.name, to);
                    }
                    FieldType::ReadonlyString {} => {
                        field.editable = false;
                    }
                    _ => {}
                }

                Ok(field)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(DocumentDataEditor { fields })
    }

    pub fn with_errors(mut self, errors: &Option<FieldValidationErrors>) -> Self {
        let errors = if let Some(errors) = errors {
            errors
        } else {
            return self;
        };

        for field in &mut self.fields {
            let mut errors: Vec<String> = errors
                .get(field.name)
                .unwrap_or(&Vec::new())
                .iter()
                .map(ToString::to_string)
                .collect();

            field.errors.append(&mut errors);
        }

        self
    }

    pub fn render(self) -> Result<String> {
        render_template(self)
    }
}
