use std::fmt;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize, de::Visitor};

use crate::entities::{Document, Id, Revision};

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct DocumentKey {
    pub id: Id,
    pub rev: Revision,
}

impl DocumentKey {
    pub fn new(id: Id, rev: Revision) -> Self {
        Self { id, rev }
    }

    pub fn for_document(document: &Document) -> Self {
        DocumentKey::new(document.id.clone(), document.rev.clone())
    }

    pub fn parse(value: &str) -> Result<Self> {
        let (id_raw, rev_raw) = value
            .split_once(' ')
            .context(anyhow!("Failed to split value '{value}'"))?;

        let id = Id::from(id_raw);
        let rev = Revision::from_safe_string(rev_raw)?;

        Ok(Self { id, rev })
    }

    pub fn serialize(&self) -> String {
        format!("{} {}", self.id, self.rev.to_safe_string())
    }
}

impl fmt::Debug for DocumentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<key: {}>", self.serialize())
    }
}

impl Serialize for DocumentKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = self.serialize();

        serializer.serialize_str(&value)
    }
}

struct DocumentKeyVisitor;

impl Visitor<'_> for DocumentKeyVisitor {
    type Value = DocumentKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string with serialized DocumentKey")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        DocumentKey::parse(value).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for DocumentKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DocumentKeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::entities::{Id, Revision};

    use super::DocumentKey;

    #[test]
    fn test_serialization() {
        let id = Id::from("123321");
        let revision = Revision::from_value(json!({"a": 1, "b": 2 })).unwrap();

        let key = DocumentKey::new(id.clone(), revision.clone());
        let serialized_key = key.serialize();
        assert_eq!(&serialized_key, "123321 a:1-b:2");

        let parsed_key = DocumentKey::parse(&serialized_key).unwrap();
        assert_eq!(parsed_key, key);
    }
}
