use std::collections::HashSet;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::{
    entities::{BLOBId, DocumentType, Id},
    markup::MarkupStr,
    schema::ATTACHMENT_TYPE,
};

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    String {},        // string
    MarkupString {},  // string
    Flag {},          // bool
    NaturalNumber {}, // u64
    // DocumentType[], empty array means any document type
    Ref(&'static [&'static str]), // string
    // DocumentType[], empty array means any document type
    RefList(&'static [&'static str]), // string[]
    BLOBId {},                        // string
    // string[], possible enum values
    Enum(&'static [&'static str]), // string
    Date {},                       // string
    Duration {},                   // string
    People {},                     // string
    Countries {},                  // string
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
    pub mandatory: bool,
    pub readonly: bool,
}

impl Field {
    #[must_use]
    pub fn could_be_title(&self) -> bool {
        matches!(self.field_type, FieldType::String {})
    }

    #[must_use]
    pub fn could_be_cover(&self) -> bool {
        matches!(self.field_type, FieldType::Ref(&[ATTACHMENT_TYPE])) && self.name == "cover"
    }

    #[must_use]
    pub fn extract_refs(&self, value: &Value) -> HashSet<Id> {
        let mut result = HashSet::new();

        match self.field_type {
            FieldType::MarkupString {} => {
                let markup: MarkupStr = value.as_str().expect("field must be string").into();

                result.extend(markup.extract_refs());
            }
            FieldType::Ref(_) => {
                let value: Id = serde_json::from_value(value.clone()).expect("field must parse");

                if !value.is_empty() {
                    result.insert(value);
                }
            }
            _ => {}
        }

        result
    }

    #[must_use]
    pub fn extract_collection_refs(&self, value: &Value) -> HashSet<Id> {
        let mut result = HashSet::new();

        if let FieldType::RefList(_) = self.field_type {
            let value: Vec<Id> = serde_json::from_value(value.clone()).expect("field must parse");

            result.extend(value);
        }

        result
    }

    #[must_use]
    pub fn extract_blob_ids(&self, value: &Value) -> HashSet<BLOBId> {
        let mut result = HashSet::new();

        if matches!(self.field_type, FieldType::BLOBId {}) {
            let value: BLOBId = serde_json::from_value(value.clone()).expect("field must parse");

            result.insert(value);
        }

        result
    }

    pub fn extract_search_data(&self, value: &Value) -> Result<Option<String>> {
        // TODO also search in Ref and RefList document titles

        match self.field_type {
            FieldType::String {} | FieldType::MarkupString {} | FieldType::People {} => value
                .as_str()
                .map(|value| Some(value.to_lowercase()))
                .ok_or_else(|| anyhow!("failed to extract field {}", self.name)),
            _ => Ok(None),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn validate(&self, value: Option<&Value>) -> Result<()> {
        let value = if let Some(value) = value {
            value
        } else {
            if self.mandatory {
                bail!("mandatory field '{}' is missing", self.name);
            }

            return Ok(());
        };

        let is_empty_string =
            value.is_string() && value.as_str().unwrap_or_default().trim().is_empty();

        if self.mandatory && is_empty_string {
            bail!("mandatory field '{}' is empty", self.name);
        }

        match self.field_type {
            FieldType::String {}
            | FieldType::MarkupString {}
            | FieldType::Ref(_)
            | FieldType::Date {}
            | FieldType::Duration {}
            | FieldType::People {}
            | FieldType::Countries {} => {
                if is_empty_string {
                    return Ok(());
                }

                if !value.is_string() {
                    bail!(
                        "field '{}' expected to be a string, got: {}",
                        self.name,
                        value
                    );
                }
            }

            FieldType::NaturalNumber {} => {
                if !value.is_number() {
                    bail!(
                        "field '{}' expected to be a number, got: {}",
                        self.name,
                        value
                    );
                }

                if value.as_u64().is_none() {
                    bail!("field '{}' expected to be a u64, got: {}", self.name, value);
                }
            }

            FieldType::Flag {} => {
                if !value.is_boolean() {
                    bail!(
                        "field '{}' expected to be a boolean, got: {}",
                        self.name,
                        value
                    );
                }
            }

            FieldType::RefList(_) => {
                let result = serde_json::from_value::<Vec<String>>(value.clone());

                if result.is_err() {
                    bail!(
                        "field '{}' expected to be a string[], got: {}",
                        self.name,
                        value
                    );
                }
            }

            FieldType::Enum(options) => {
                if is_empty_string {
                    return Ok(());
                }

                if !value.is_string() {
                    bail!(
                        "field '{}' expected to be a string, got: {}",
                        self.name,
                        value
                    );
                }

                if !options.contains(&value.as_str().unwrap_or_default()) {
                    bail!(
                        "field '{}' is {}, expected to be one of {}",
                        self.name,
                        value,
                        options.join(", ")
                    );
                }
            }

            FieldType::BLOBId {} => {
                if is_empty_string {
                    return Ok(());
                }

                if !value.is_string() {
                    bail!(
                        "field '{}' expected to be a string, got: {}",
                        self.name,
                        value
                    );
                }

                let blob_id = value.as_str().unwrap_or_default();

                BLOBId::is_valid_blob_id(blob_id).context(anyhow!(
                    "field '{}' expected to be a valid BLOB id",
                    self.name
                ))?;
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn get_expected_ref_types(&self) -> Option<&[&str]> {
        match self.field_type {
            FieldType::Ref(document_types) | FieldType::RefList(document_types) => {
                Some(document_types)
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn can_collect(&self, document_type: &DocumentType) -> bool {
        match self.field_type {
            FieldType::RefList(ref_types) => {
                ref_types.is_empty() || ref_types.contains(&document_type.as_ref())
            }
            _ => false,
        }
    }
}
