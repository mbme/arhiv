use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDescription {
    pub document_type: String,
    pub collection_of: Option<Collection>,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Serialize, Deserialize)]
pub enum FieldType {
    String,
    MarkupString,
    Ref(String),
    Enum(Vec<String>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub item_type: String,
    pub item_field_name: String,
}
