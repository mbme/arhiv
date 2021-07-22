use anyhow::*;
use serde::Serialize;

use arhiv_core::{
    entities::Document,
    markup::MarkupStr,
    schema::{extract_ids_from_reflist, DataDescription, FieldType},
    Arhiv,
};

use crate::{components::Ref, markup::MarkupStringExt};

#[derive(Serialize)]
pub enum FieldKind {
    Title,
    Markup,
    Html,
    String,
}

#[derive(Serialize)]
pub struct Field {
    name: &'static str,
    value: String,
    kind: FieldKind,
}

impl Field {
    pub fn empty(name: &'static str) -> Self {
        Field {
            name,
            value: "".to_string(),
            kind: FieldKind::Html,
        }
    }
}

pub fn prepare_fields(
    document: &Document,
    arhiv: &Arhiv,
    data_description: &DataDescription,
) -> Result<Vec<Field>> {
    let title_field = arhiv
        .schema
        .get_data_description(&document.document_type)?
        .pick_title_field()?;

    data_description
        .fields
        .iter()
        .map(|field_description| {
            let is_title = field_description.name == title_field.name;

            match &field_description.field_type {
                FieldType::MarkupString {} => {
                    let markup: MarkupStr = document
                        .data
                        .get_str(field_description.name)
                        .unwrap_or_default()
                        .into();

                    Ok(Field {
                        name: field_description.name,
                        value: markup.to_html(arhiv),
                        kind: FieldKind::Markup,
                    })
                }
                FieldType::Ref(_) => {
                    if let Some(value) = document.data.get_str(field_description.name) {
                        Ok(Field {
                            name: field_description.name,
                            value: Ref::from_id(value).preview_attachments().render(arhiv)?,
                            kind: FieldKind::Html,
                        })
                    } else {
                        Ok(Field::empty(field_description.name))
                    }
                }
                FieldType::RefList(_) => {
                    let value = document
                        .data
                        .get_str(field_description.name)
                        .unwrap_or_default();
                    let ids = extract_ids_from_reflist(value);

                    Ok(Field {
                        name: field_description.name,
                        value: ids
                            .into_iter()
                            .map(|item| Ref::from_id(item).render(arhiv))
                            .collect::<Result<Vec<_>>>()?
                            .join("\n"),
                        kind: FieldKind::Html,
                    })
                }
                FieldType::Flag {} => {
                    let value = document
                        .data
                        .get(field_description.name)
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);

                    Ok(Field {
                        name: field_description.name,
                        value: value.to_string(),
                        kind: FieldKind::String,
                    })
                }
                _ => {
                    let value = document
                        .data
                        .get_str(field_description.name)
                        .unwrap_or_default()
                        .to_string();

                    Ok(Field {
                        name: field_description.name,
                        value,
                        kind: if is_title {
                            FieldKind::Title
                        } else {
                            FieldKind::String
                        },
                    })
                }
            }
        })
        .collect::<Result<Vec<_>>>()
}
