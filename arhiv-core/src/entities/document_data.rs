use std::convert::TryInto;

use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DocumentData(Map<String, Value>);

impl DocumentData {
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

    pub fn get(&self, field: &str) -> Option<&Value> {
        let value = self.0.get(field)?;

        if value.is_null() {
            None
        } else {
            Some(value)
        }
    }

    pub fn get_str<'doc>(&self, field: &str) -> Option<&str> {
        let value = self.get(field)?;

        Some(
            value
                .as_str()
                .expect(&format!("can't use field '{}' as &str", field)),
        )
    }

    pub fn get_mandatory_str(&self, field: &str) -> &str {
        self.get_str(field)
            .expect(&format!("str field '{}' must be present", field))
    }

    pub fn get_bool(&self, field: &str) -> Option<bool> {
        self.get(field).map(|value| value.as_bool()).flatten()
    }

    pub fn get_number(&self, field: &str) -> Option<u64> {
        self.get(field).map(|value| value.as_u64()).flatten()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self.0).expect("failed to serialize DocumentData")
    }
}

impl Into<Value> for DocumentData {
    fn into(self) -> Value {
        Value::Object(self.0)
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
