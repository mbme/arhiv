use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSchema {
    pub version: u8,
    pub modules: Vec<DataDescription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataDescription {
    pub document_type: String,
    pub collection_of: Option<Collection>,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FieldType {
    String {},
    MarkupString {},
    Ref(String),
    Enum(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub item_type: String,
}
