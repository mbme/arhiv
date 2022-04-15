use anyhow::Result;
use serde::Serialize;
use serde_json::json;

use arhiv_core::{entities::Document, markup::MarkupStr, schema::FieldType, Arhiv};

use crate::{components::Ref, markup::MarkupStringExt, template_fn};

template_fn!(render_template, "./document_data_viewer.html.tera");

#[derive(Serialize)]
enum FieldKind {
    Title,
    Markup,
    Html,
    String,
}

#[derive(Serialize)]
struct Field {
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

pub struct DocumentDataViewer<'d> {
    document: &'d Document,
}

impl<'d> DocumentDataViewer<'d> {
    pub fn new(document: &'d Document) -> Self {
        DocumentDataViewer { document }
    }

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let data_description = arhiv
            .get_schema()
            .get_data_description(&self.document.document_type)?;

        let title_field = data_description.pick_title_field(&self.document.subtype);

        let data = &self.document.data;

        let fields = data_description
            .iter_fields(&self.document.subtype)
            .map(|field_description| {
                let is_title = title_field.map_or(false, |title_field| {
                    title_field.name == field_description.name
                });

                match &field_description.field_type {
                    FieldType::MarkupString {} => {
                        let markup: MarkupStr = data
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
                        if let Some(value) = data.get_str(field_description.name) {
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
                        if let Some(value) = data.get(field_description.name) {
                            let ids: Vec<String> = serde_json::from_value(value.clone())?;

                            Ok(Field {
                                name: field_description.name,
                                value: ids
                                    .into_iter()
                                    .map(|item| Ref::from_id(item).render(arhiv))
                                    .collect::<Result<Vec<_>>>()?
                                    .join("\n"),
                                kind: FieldKind::Html,
                            })
                        } else {
                            Ok(Field::empty(field_description.name))
                        }
                    }
                    FieldType::Flag {} => {
                        let value = data.get_bool(field_description.name).unwrap_or(false);

                        Ok(Field {
                            name: field_description.name,
                            value: value.to_string(),
                            kind: FieldKind::String,
                        })
                    }
                    FieldType::NaturalNumber {} => {
                        let value = data.get_number(field_description.name);

                        Ok(Field {
                            name: field_description.name,
                            value: value.map(|value| value.to_string()).unwrap_or_default(),
                            kind: FieldKind::String,
                        })
                    }
                    _ => {
                        let value = data
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
            .collect::<Result<Vec<_>>>()?;

        let has_empty_fields = fields.iter().find(|field| field.value.is_empty());

        render_template(json!({
            "id": self.document.id,
            "document_type": self.document.document_type,
            "subtype": self.document.subtype,
            "updated_at": self.document.updated_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string(),
            "show_empty_fields_toggle": has_empty_fields,
            "fields": fields,
        }))
    }
}
