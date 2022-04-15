use anyhow::Result;
use serde::Serialize;

use arhiv_core::{
    entities::DocumentData,
    schema::{DataDescription, FieldType},
    ValidationError,
};
use serde_json::json;

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
    for_subtypes: &'static [&'static str],
}

pub struct DocumentDataEditor {
    fields: Vec<FormField>,
    subtype: String,
    subtypes: &'static [&'static str],
    errors: Vec<String>,
}

impl DocumentDataEditor {
    pub fn new(
        subtype: &str,
        data: &DocumentData,
        data_description: &DataDescription,
    ) -> Result<Self> {
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
                    editable: !field.readonly,
                    errors: vec![],
                    for_subtypes: field
                        .for_subtypes
                        .unwrap_or_else(|| data_description.subtypes.unwrap_or(&[""])),
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

        Ok(DocumentDataEditor {
            fields,
            subtype: subtype.to_string(),
            subtypes: data_description.subtypes.unwrap_or(&[""]),
            errors: vec![],
        })
    }

    pub fn with_validation_error(mut self, error: Option<ValidationError>) -> Self {
        match error {
            Some(ValidationError::DocumentError { mut errors }) => {
                self.errors.append(&mut errors);
            }
            Some(ValidationError::FieldError { errors }) => {
                for field in &mut self.fields {
                    let mut errors: Vec<String> = errors
                        .get(field.name)
                        .unwrap_or(&Vec::new())
                        .iter()
                        .map(ToString::to_string)
                        .collect();

                    field.errors.append(&mut errors);
                }
            }
            None => {}
        };

        self
    }

    pub fn render(self, action_url: &str, cancel_url: &str) -> Result<String> {
        render_template(json!({
            "subtype": self.subtype,
            "subtypes": self.subtypes,
            "errors": self.errors,
            "fields": self.fields,
            "action_url": action_url,
            "cancel_url": cancel_url,
        }))
    }
}
