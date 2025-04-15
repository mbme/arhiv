use anyhow::{ensure, Context, Result};

use crate::{
    entities::{parse_string_vec, Document},
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

        let first_doc = originals.first().expect("Originals can't be empty");
        let first_id = &first_doc.id;
        let first_type = &first_doc.document_type;

        ensure!(
            originals
                .iter()
                .all(|doc| &doc.id == first_id && &doc.document_type == first_type),
            "All documents must have the same id and document_type"
        );

        if let Some(base_doc) = &base {
            let base_id = &base_doc.id;
            let base_type = &base_doc.document_type;

            ensure!(
                first_id == base_id && first_type == base_type,
                "All documents must have the same id and document_type"
            );
        }

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

        while let Some(doc_b) = originals.next() {
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
        let a_is_older = doc_a.updated_at < doc_b.updated_at;

        let mut result = doc_a.clone();

        for field in self.schema.iter_fields(&doc_a.document_type)? {
            let value_base = base.and_then(|base| base.data.get(field.name));
            let value_a = doc_a.data.get(field.name);
            let value_b = doc_a.data.get(field.name);

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
                    // value_a and value_b are equal, no nothing
                    if value_a == value_b {
                        continue;
                    }

                    (value_a, value_b)
                }
                (None, None, None) | (Some(_), None, None) => {
                    // value_a and value_b are equal, no nothing
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

                    let resulting_value = self.merge_strings(value_base, value_a, value_b)?;
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

                    let resulting_value = self.merge_string_vec(value_base, value_a, value_b)?;
                    result.data.set(field.name, resulting_value);
                }

                // Last Write Wins
                FieldType::Flag {}
                | FieldType::NaturalNumber {}
                | FieldType::Ref(_)
                | FieldType::Enum(_)
                | FieldType::Date {}
                | FieldType::Duration {} => {
                    result
                        .data
                        .set(field.name, if a_is_older { value_b } else { value_a });
                }
            }
        }

        Ok(result)
    }

    fn merge_strings(&self, base: Option<&str>, doc_a: &str, doc_b: &str) -> Result<String> {
        // Merge strings
        // if there's no base, merge pairwise
        // else 3-way merge using base
        todo!()
    }

    fn merge_string_vec(
        &self,
        base: Option<Vec<&str>>,
        doc_a: Vec<&str>,
        doc_b: Vec<&str>,
    ) -> Result<Vec<&str>> {
        todo!()
    }
}
