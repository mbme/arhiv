use std::fmt;

use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::Id;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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
    pub fn get_mut(&mut self, field: &str) -> Option<&mut Value> {
        let value = self.0.get_mut(field)?;

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
                .unwrap_or_else(|| panic!("can't use field '{field}' as &str")),
        )
    }

    #[must_use]
    pub fn get_mandatory_str(&self, field: &str) -> &str {
        self.get_str(field)
            .unwrap_or_else(|| panic!("str field '{field}' must be present"))
    }

    #[must_use]
    pub fn get_bool(&self, field: &str) -> Option<bool> {
        // FIXME this must return a Result<Option<bool>>
        self.get(field).and_then(serde_json::Value::as_bool)
    }

    #[must_use]
    pub fn get_number(&self, field: &str) -> Option<u64> {
        self.get(field).and_then(serde_json::Value::as_u64)
    }

    pub fn get_ref_list(&self, field: &str) -> Result<Option<Vec<&str>>> {
        let value = if let Some(value) = self.get(field) {
            value
        } else {
            return Ok(None);
        };

        let arr = if let Some(value) = value.as_array() {
            value
        } else {
            bail!("Field '{field}' expected to be an array")
        };

        let arr = arr
            .iter()
            .map(|value| {
                value.as_str().context(anyhow!(
                    "Field '{field}' expected to be an array of strings"
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Some(arr))
    }

    pub fn add_to_ref_list(&mut self, field: &str, id: &Id) -> Result<()> {
        if let Some(value) = self.get_mut(field) {
            let ref_list = value
                .as_array_mut()
                .context(anyhow!("field {field} isn't an array"))?;

            let id = serde_json::to_value(id).context("failed to serialize id")?;

            ensure!(
                !ref_list.contains(&id),
                "field {field} of type RefList already contains id {id}"
            );

            ref_list.push(id);
        } else {
            self.set(field, vec![id]);
        }

        Ok(())
    }

    pub fn remove_from_ref_list(&mut self, field: &str, id: &Id) -> Result<()> {
        let value = self.get_mut(field).context("field {field} is missing")?;

        let ref_list = value
            .as_array_mut()
            .context(anyhow!("field {field} isn't an array"))?;

        let id = serde_json::to_value(id).context("failed to serialize id")?;

        let pos = ref_list
            .iter()
            .position(|item| item == &id)
            .context("field {field} of type RefList doesn't contain id {id}")?;

        ref_list.remove(pos);

        Ok(())
    }

    pub fn iter_fields(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::entities::Id;

    use super::DocumentData;

    #[test]
    fn test_add_to_ref_list() -> Result<()> {
        let mut data = DocumentData::new();

        let id = Id::new();
        data.add_to_ref_list("list", &id)?;

        // can't add the same id
        assert!(data.add_to_ref_list("list", &id).is_err());

        assert_eq!(
            data.get_ref_list("list")?.unwrap(),
            vec![id.to_string().as_str()]
        );

        Ok(())
    }

    #[test]
    fn test_remove_from_ref_list() -> Result<()> {
        let mut data = DocumentData::new();

        let id = Id::new();

        data.add_to_ref_list("list", &id)?;

        data.remove_from_ref_list("list", &id)?;

        // can't remove the same id
        assert!(data.remove_from_ref_list("list", &id).is_err());

        assert_eq!(data.get_ref_list("list")?.unwrap(), Vec::<&str>::new());

        Ok(())
    }
}
