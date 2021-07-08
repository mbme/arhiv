use anyhow::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    String {},
    NaturalNumber {},
    MarkupString {},
    Ref(&'static str),
    Enum(Vec<&'static str>),
    ISBN {},
    Date {},
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
    pub optional: bool,
}

impl Field {
    pub fn get_enum_values(&self) -> Result<&Vec<&'static str>> {
        match self.field_type {
            FieldType::Enum(ref values) => Ok(values),
            _ => bail!("field {} isn't enum", self.name),
        }
    }
}
