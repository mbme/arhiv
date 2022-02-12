use std::fmt;

use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DocumentData(Map<String, Value>);

impl DocumentData {
    #[must_use]
    pub fn new() -> Self {
        DocumentData(Map::new())
    }

    pub fn set(&mut self, field: impl Into<String>, value: impl Serialize) {
        self.0.insert(
            field.into(),
            serde_json::to_value(value).expect("failed to serialize value"),
        );
    }

    pub fn remove(&mut self, field: impl AsRef<str>) {
        self.0.remove(field.as_ref());
    }

    pub fn rename(&mut self, field: impl AsRef<str>, new_field: impl AsRef<str>) {
        let field = field.as_ref();
        let new_field = new_field.as_ref();

        if let Some(value) = self.0.remove(field) {
            self.set(new_field, value);
        }
    }

    #[must_use]
    pub fn get(&self, field: &str) -> Option<&Value> {
        let value = self.0.get(field)?;

        if value.is_null() {
            None
        } else {
            Some(value)
        }
    }

    #[must_use]
    pub fn get_str(&self, field: &str) -> Option<&str> {
        let value = self.get(field)?;

        Some(
            value
                .as_str()
                .unwrap_or_else(|| panic!("can't use field '{}' as &str", field)),
        )
    }

    #[must_use]
    pub fn get_mandatory_str(&self, field: &str) -> &str {
        self.get_str(field)
            .unwrap_or_else(|| panic!("str field '{}' must be present", field))
    }

    #[must_use]
    pub fn get_bool(&self, field: &str) -> Option<bool> {
        self.get(field).and_then(serde_json::Value::as_bool)
    }

    #[must_use]
    pub fn get_number(&self, field: &str) -> Option<u64> {
        self.get(field).and_then(serde_json::Value::as_u64)
    }
}

impl fmt::Display for DocumentData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self.0).expect("failed to serialize DocumentData")
        )
    }
}

impl Default for DocumentData {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DocumentData> for Value {
    fn from(val: DocumentData) -> Self {
        Value::Object(val.0)
    }
}

impl TryInto<DocumentData> for Value {
    type Error = Error;

    fn try_into(self) -> Result<DocumentData, Self::Error> {
        match self {
            Value::Object(value) => Ok(DocumentData(value)),
            _ => bail!("failed to convert into DocumentData: Value is not an object"),
        }
    }
}
