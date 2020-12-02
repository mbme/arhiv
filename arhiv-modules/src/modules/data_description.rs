use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDescription {
    pub document_type: String,
    pub collection_of: Option<Collection>,
    pub fields: HashMap<String, Field>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub field_type: FieldType,
}

#[derive(Serialize, Deserialize)]
pub enum FieldType {
    String,
    MarkupString,
    Ref,
    Enum(Vec<String>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub item_type: String,
    pub field_name: String,
}
