use anyhow::*;
use serde::Serialize;

use crate::entities::Id;

#[derive(Serialize, Debug, Clone)]
pub enum FieldType {
    String {},
    NaturalNumber {},
    MarkupString {},
    Flag {},
    Ref(&'static str),
    RefList(&'static str),
    Enum(Vec<&'static str>),
    ISBN {},
    Date {},
    Duration {},
    People {},
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: &'static str,
    pub field_type: FieldType,
    pub optional: bool,
}

impl Field {
    pub fn get_enum_values(&self) -> Result<&Vec<&'static str>> {
        match self.field_type {
            FieldType::Enum(ref values) => Ok(values),
            _ => bail!("field {} isn't enum", self.name),
        }
    }
}

pub fn extract_ids_from_reflist(reflist: &str) -> Vec<Id> {
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
    use crate::schema::extract_ids_from_reflist;

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
