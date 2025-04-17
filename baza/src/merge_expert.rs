use std::collections::HashSet;

use anyhow::{ensure, Context, Result};

use rs_utils::merge::{merge_slices_three_way, merge_strings_three_way};

use crate::{
    entities::{parse_string_vec, Document, DocumentType},
    schema::{DataSchema, FieldType},
};

pub struct MergeExpert {
    schema: DataSchema,
}

impl MergeExpert {
    pub fn new(schema: DataSchema) -> Self {
        MergeExpert { schema }
    }

    fn validate_args(&self, base: &Option<Document>, originals: &Vec<&Document>) -> Result<()> {
        ensure!(
            originals.len() > 1,
            "Expected at least 2 documents to merge, got {}",
            originals.len()
        );

        let mut all_ids = originals.iter().map(|doc| &doc.id).collect::<HashSet<_>>();

        let mut all_doc_types = originals
            .iter()
            .map(|doc| &doc.document_type)
            .collect::<HashSet<_>>();

        if let Some(base_doc) = &base {
            all_ids.insert(&base_doc.id);
            all_doc_types.insert(&base_doc.document_type);
        }

        all_doc_types.remove(&DocumentType::erased());

        ensure!(
            all_ids.len() == 1,
            "All documents must have the same id, got {all_ids:?}"
        );
        ensure!(
            all_doc_types.len() < 2,
            "All documents must have the same document_type, got {all_doc_types:?}"
        );

        Ok(())
    }

    pub fn merge_originals(
        &self,
        base: Option<Document>,
        mut originals: Vec<&Document>,
    ) -> Result<Document> {
        self.validate_args(&base, &originals)?;

        // sort from oldest to newest
        originals.sort_by_key(|document| document.updated_at);

        // if all erased, just use the oldest one
        let all_erased = originals.iter().all(|document| document.is_erased());
        if all_erased {
            return Ok(originals[0].clone());
        }

        // if there are erased & non-erased, use only non-erased
        originals.retain(|document| !document.is_erased());

        // there was only one non-erased, use it
        if originals.len() == 1 {
            return Ok(originals[0].clone());
        }

        let mut originals = originals.into_iter();

        let mut doc_a = originals.next().expect("Originals can't be empty").clone();

        for doc_b in originals {
            doc_a = self.merge_documents(base.as_ref(), &doc_a, doc_b)?;
        }

        Ok(doc_a)
    }

    fn merge_documents(
        &self,
        base: Option<&Document>,
        doc_a: &Document,
        doc_b: &Document,
    ) -> Result<Document> {
        let mut result = doc_a.clone();

        for field in self.schema.iter_fields(&doc_a.document_type)? {
            let value_base = base.and_then(|base| base.data.get(field.name));
            let value_a = doc_a.data.get(field.name);
            let value_b = doc_b.data.get(field.name);

            // handle cases when field values are equal, or when there's an explicit resolution
            let (value_a, value_b) = match (value_base, value_a, value_b) {
                (None, Some(value_a), None) => {
                    result.data.set(field.name, value_a);
                    continue;
                }
                (None, None, Some(value_b)) => {
                    result.data.set(field.name, value_b);
                    continue;
                }
                (Some(value_base), Some(value_a), None) => {
                    if value_base == value_a {
                        result.data.remove(field.name);
                    } else {
                        result.data.set(field.name, value_a);
                    }

                    continue;
                }
                (Some(value_base), None, Some(value_b)) => {
                    if value_base == value_b {
                        result.data.remove(field.name);
                    } else {
                        result.data.set(field.name, value_b);
                    }

                    continue;
                }
                (_, Some(value_a), Some(value_b)) => {
                    // value_a and value_b are equal, do nothing
                    if value_a == value_b {
                        continue;
                    }

                    (value_a, value_b)
                }
                (None, None, None) | (Some(_), None, None) => {
                    // value_a and value_b are equal, do nothing
                    continue;
                }
            };

            match field.field_type {
                FieldType::String {}
                | FieldType::People {}
                | FieldType::Countries {}
                | FieldType::MarkupString {} => {
                    let value_base = value_base
                        .map(|value_base| {
                            value_base
                                .as_str()
                                .context("Expected value_base to be a string")
                        })
                        .transpose()?;

                    let value_a = value_a
                        .as_str()
                        .context("Expected value_a to be a string")?;

                    let value_b = value_b
                        .as_str()
                        .context("Expected value_b to be a string")?;

                    let resulting_value =
                        merge_strings_three_way(value_base.unwrap_or_default(), value_a, value_b);

                    result.data.set(field.name, resulting_value);
                }

                // Merge string arrays
                FieldType::RefList(_) => {
                    let value_base = value_base
                        .map(|value_base| {
                            parse_string_vec(value_base)
                                .context("Failed to use value_base as Vec<&str>")
                        })
                        .transpose()?;

                    let value_a =
                        parse_string_vec(value_a).context("Failed to use value_a as Vec<&str>")?;

                    let value_b =
                        parse_string_vec(value_b).context("Failed to use value_b as Vec<&str>")?;

                    let resulting_value = merge_slices_three_way(
                        value_base.unwrap_or_default().as_slice(),
                        &value_a,
                        &value_b,
                    );

                    result.data.set(field.name, resulting_value);
                }

                // Last Write Wins
                FieldType::Flag {}
                | FieldType::NaturalNumber {}
                | FieldType::Ref(_)
                | FieldType::Enum(_)
                | FieldType::Date {}
                | FieldType::Duration {} => {
                    result.data.set(field.name, value_b);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::{
        entities::new_document,
        schema::{DataDescription, Field},
    };

    use super::*;

    fn assert_schema_merge_result(
        schema: DataSchema,
        base: Value,
        originals: Vec<Value>,
        result_data: Value,
    ) {
        let expert = MergeExpert::new(schema);

        let base_doc = new_document(base);
        let docs: Vec<_> = originals
            .into_iter()
            .map(|value| base_doc.clone().with_data(value))
            .collect();
        let refs: Vec<&_> = docs.iter().collect();

        let result = expert
            .merge_originals(Some(base_doc.clone()), refs)
            .unwrap();

        assert_eq!(result.data, new_document(result_data).data);
    }

    fn assert_merge_result(base: Value, originals: Vec<Value>, result_data: Value) {
        let schema = DataSchema::new_test_schema();

        assert_schema_merge_result(schema, base, originals, result_data);
    }

    #[test]
    fn test_merge_erased() {
        assert_merge_result(
            json!({ "test": "base" }),
            vec![
                Value::Null, //
                json!({ "test": "base 2" }),
            ],
            json!({ "test": "base 2"}),
        );

        assert_merge_result(
            json!({ "test": "base" }),
            vec![
                Value::Null, //
                Value::Null,
            ],
            Value::Null,
        );

        assert_merge_result(
            Value::Null,
            vec![
                Value::Null, //
                json!({ "test": "base 2" }),
            ],
            json!({ "test": "base 2"}),
        );
    }

    #[test]
    fn test_merge_last_write_wins() {
        assert_merge_result(
            json!({ "ref": "base" }),
            vec![
                json!({ "ref": "first" }), //
                json!({ "ref": "second" }),
            ],
            json!({ "ref": "second"}),
        );
    }

    #[test]
    fn test_merge_ref_list() {
        let schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "test_type",
                title_format: "title",
                fields: vec![Field {
                    name: "refs",
                    field_type: FieldType::RefList(&["*"]),
                    mandatory: false,
                    readonly: false,
                }],
            }],
        );

        assert_schema_merge_result(
            schema,
            json!({ "refs": &["base"] }),
            vec![
                json!({ "refs": &["first", "base"] }),
                json!({ "refs": &["base", "second"] }),
            ],
            json!({ "refs": &["first", "base", "second"]}),
        );
    }

    #[test]
    fn test_merge_multiple() {
        assert_merge_result(
            json!({ "test": "base" }),
            vec![
                json!({ "test": "base left" }),
                json!({ "test": "base right" }),
                json!({ "test": "base third" }),
            ],
            json!({ "test": "base left right third"}),
        );
    }

    #[test]
    fn test_merge_originals() {
        assert_merge_result(
            json!({ "test": "base" }),
            vec![json!({ "test": "base left" }), json!({})],
            json!({ "test": "base left"}),
        );

        assert_merge_result(
            json!({ "test": "base" }),
            vec![json!({ "test": "base" }), json!({})],
            json!({}),
        );

        assert_merge_result(
            json!({ "test": "base" }),
            vec![json!({}), json!({})],
            json!({}),
        );

        assert_merge_result(
            json!({}),
            vec![json!({ "test": "left" }), json!({})],
            json!({ "test": "left" }),
        );

        // merge conflict
        assert_merge_result(
            json!({ "test": "base" }),
            vec![
                json!({ "test": "base left" }),
                json!({ "test": "base right" }),
            ],
            json!({ "test": "base left right"}),
        );
    }
}
