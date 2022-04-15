use std::collections::HashSet;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::{
    entities::{BLOBId, Id},
    markup::MarkupStr,
};

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    String {},                     // string
    MarkupString {},               // string
    Flag {},                       // bool
    NaturalNumber {},              // u64
    Ref(&'static str),             // string
    RefList(&'static str),         // string[]
    BLOBId,                        // string
    Enum(&'static [&'static str]), // string
    Date {},                       // string
    Duration {},                   // string
    People {},                     // string
    Countries {},                  // string
}

#[derive(Serialize, Debug, Clone)]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
    pub mandatory: bool,
    pub readonly: bool,
    pub for_subtypes: Option<&'static [&'static str]>,
}

impl Field {
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

                result.insert(value);
            }
            FieldType::RefList(_) => {
                let value: Vec<Id> =
                    serde_json::from_value(value.clone()).expect("field must parse");

                result.extend(value);
            }
            _ => {}
        }

        result
    }

    #[must_use]
    pub fn extract_blob_ids(&self, value: &Value) -> HashSet<BLOBId> {
        let mut result = HashSet::new();

        if matches!(self.field_type, FieldType::BLOBId) {
            let value: BLOBId = serde_json::from_value(value.clone()).expect("field must parse");

            result.insert(value);
        }

        result
    }

    pub fn from_raw_value(&self, raw_value: Option<&str>) -> Result<Value> {
        match self.field_type {
            // skip empty string from ref field
            FieldType::Ref(_) => {
                let value = raw_value.unwrap_or_default().trim();

                if value.is_empty() {
                    return Ok(Value::Null);
                }

                serde_json::to_value(value).context("failed to serialize")
            }

            // convert string list of refs into array of ids
            FieldType::RefList(_) => {
                let value = extract_ids_from_reflist(raw_value.unwrap_or_default());

                serde_json::to_value(value).context("failed to serialize")
            }

            // convert string "true" to boolean, default to false
            // (since browsers do not send values of unchecked radio inputs)
            FieldType::Flag {} => {
                let value = raw_value.unwrap_or_default() == "true";

                serde_json::to_value(value).context("failed to serialize")
            }

            // convert string to number
            FieldType::NaturalNumber {} => {
                let raw_value = raw_value.unwrap_or_default().trim();

                if raw_value.is_empty() {
                    return Ok(Value::Null);
                }

                let value: u64 = raw_value.parse().context(anyhow!(
                    "failed to parse natural number field {}: {}",
                    self.name,
                    raw_value
                ))?;

                serde_json::to_value(value).context("failed to serialize")
            }

            _ => serde_json::to_value(raw_value).context("failed to serialize"),
        }
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

            FieldType::BLOBId => {
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
    pub fn get_expected_ref_type(&self) -> Option<&str> {
        match self.field_type {
            FieldType::Ref(document_type) | FieldType::RefList(document_type) => {
                Some(document_type)
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn for_subtype(&self, subtype: &str) -> bool {
        if let Some(for_subtypes) = self.for_subtypes {
            for_subtypes.contains(&subtype)
        } else {
            true
        }
    }
}

fn extract_ids_from_reflist(reflist: &str) -> Vec<Id> {
    reflist
        .replace(',', " ")
        .split(' ')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(Into::into)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::extract_ids_from_reflist;

    #[test]
    fn test_extract_ids_from_reflist() {
        assert_eq!(
            extract_ids_from_reflist(""), //
            vec![],
        );

        assert_eq!(
            extract_ids_from_reflist("test"), //
            vec!["test".into()],
        );

        assert_eq!(
            extract_ids_from_reflist("test,123"), //
            vec!["test".into(), "123".into()],
        );

        assert_eq!(
            extract_ids_from_reflist("test , 123"), //
            vec!["test".into(), "123".into()],
        );

        assert_eq!(
            extract_ids_from_reflist("test 123 ,,, ,"), //
            vec!["test".into(), "123".into()],
        );
    }
}
