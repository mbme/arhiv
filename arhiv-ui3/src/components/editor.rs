use anyhow::*;
use serde::Serialize;

use crate::{template_fn, urls::document_url};
use arhiv_core::{
    entities::{Document, Id},
    schema::{DataDescription, FieldType},
};

template_fn!(render_template, "./editor.html.tera");

#[derive(Serialize)]
struct FormField {
    name: &'static str,
    label: String,
    field_type: FieldType,
    mandatory: bool,
    value: String,
}

#[derive(Serialize)]
pub struct Editor<'d> {
    fields: Vec<FormField>,
    document: &'d Document,
    url_on_save: String,
}

impl<'d> Editor<'d> {
    pub fn new(
        document: &'d Document,
        data_description: &DataDescription,
        parent_collection: &Option<Id>,
    ) -> Result<Self> {
        let fields = data_description
            .fields
            .iter()
            .map(|field| {
                let value = match &field.field_type {
                    FieldType::Flag {} => document
                        .data
                        .get_bool(field.name)
                        .unwrap_or(false)
                        .to_string(),

                    FieldType::NaturalNumber {} => document
                        .data
                        .get_number(field.name)
                        .map(|value| value.to_string())
                        .unwrap_or_default(),

                    FieldType::RefList(_) => {
                        let value = document.data.get(field.name);

                        if let Some(value) = value {
                            let refs: Vec<String> = serde_json::from_value(value.clone())?;

                            refs.join(" ")
                        } else {
                            "".to_string()
                        }
                    }

                    _ => document
                        .data
                        .get_str(field.name)
                        .unwrap_or_default()
                        .to_string(),
                };

                let mut field = FormField {
                    name: field.name,
                    label: field.name.to_string(),
                    field_type: field.field_type.clone(),
                    mandatory: field.mandatory,
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

        let url_on_save = document_url(&document.id, parent_collection);

        Ok(Editor {
            fields,
            document,
            url_on_save,
        })
    }

    pub fn render(self) -> Result<String> {
        render_template(self)
    }
}
