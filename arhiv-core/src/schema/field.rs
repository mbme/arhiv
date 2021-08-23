use std::collections::HashSet;

use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use crate::{entities::Id, markup::MarkupStr};

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    String {},               // string
    MarkupString {},         // string
    Flag {},                 // bool
    NaturalNumber {},        // u64
    Ref(&'static str),       // string
    RefList(&'static str),   // string[]
    Enum(Vec<&'static str>), // string
    ISBN {},                 // string
    Date {},                 // string
    Duration {},             // string
    People {},               // string
    Countries {},            // string
}

#[derive(Serialize, Debug, Clone)]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
    pub mandatory: bool,
}

impl Field {
    pub fn get_enum_values(&self) -> Result<&Vec<&'static str>> {
        match self.field_type {
            FieldType::Enum(ref values) => Ok(values),
            _ => bail!("field {} isn't enum", self.name),
        }
    }

    pub fn get_refs(&self, value: &Value) -> HashSet<Id> {
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

    pub fn from_string(&self, raw_value: &str) -> Result<Value> {
        let raw_value = raw_value.trim();

        match self.field_type {
            // skip empty string from ref field
            FieldType::Ref(_) => {
                if raw_value.is_empty() {
                    return Ok(Value::Null);
                } else {
                    return serde_json::to_value(raw_value).context("failed to serialize");
                }
            }

            // convert string list of refs into array of ids
            FieldType::RefList(_) => {
                let value = extract_ids_from_reflist(&raw_value);

                return serde_json::to_value(value).context("failed to serialize");
            }

            // convert string "true" to boolean
            FieldType::Flag {} => {
                let value = raw_value == "true";

                return serde_json::to_value(value).context("failed to serialize");
            }

            // convert string to number
            FieldType::NaturalNumber {} => {
                if raw_value.is_empty() {
                    return Ok(Value::Null);
                }

                let value: u64 = raw_value.parse().context(anyhow!(
                    "failed to parse natural number field {}: {}",
                    self.name,
                    raw_value
                ))?;

                return serde_json::to_value(value).context("failed to serialize");
            }

            _ => {
                return serde_json::to_value(raw_value).context("failed to serialize");
            }
        };
    }

    pub fn extract_search_data(&self, value: &Value) -> Result<Option<String>> {
        // FIXME also search in Ref and RefList document titles

        match self.field_type {
            FieldType::String {} | FieldType::MarkupString {} | FieldType::People {} => value
                .as_str()
                .map(|value| Some(value.to_lowercase()))
                .ok_or(anyhow!("failed to extract field {}", self.name)),
            _ => Ok(None),
        }
    }

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
            | FieldType::ISBN {}
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

            FieldType::Enum(ref options) => {
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
        }

        Ok(())
    }
}

fn extract_ids_from_reflist(reflist: &str) -> Vec<Id> {
    reflist
        .replace(",", " ")
        .split(" ")
        .map(|item| item.trim())
        .filter(|item| item.len() > 0)
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
