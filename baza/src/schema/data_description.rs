use serde::Serialize;

use super::field::Field;

#[derive(Serialize, Debug, Clone)]
pub struct DataDescription {
    pub document_type: &'static str,
    pub subtypes: Option<&'static [&'static str]>,
    pub fields: Vec<Field>,
}

impl DataDescription {
    pub fn get_field(&self, name: impl AsRef<str>) -> Option<&Field> {
        let name = name.as_ref();

        self.fields.iter().find(|field| field.name == name)
    }

    #[must_use]
    pub fn is_supported_subtype(&self, subtype: &str) -> bool {
        self.subtypes.unwrap_or(&[""]).contains(&subtype)
    }
}
