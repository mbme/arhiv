use serde::Serialize;

use super::field::Field;

#[derive(Serialize, Debug, Clone)]
pub struct DataDescription {
    pub document_type: &'static str,
    pub fields: Vec<Field>,
    pub title_format: &'static str, // https://docs.rs/tinytemplate/latest/tinytemplate/syntax/
}

impl DataDescription {
    pub fn get_field(&self, name: impl AsRef<str>) -> Option<&Field> {
        let name = name.as_ref();

        self.fields.iter().find(|field| field.name == name)
    }
}
