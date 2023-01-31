use anyhow::{anyhow, Result};

use crate::{
    entities::{DocumentData, DocumentType, Refs},
    schema::{DataSchema, Field},
    search::MultiSearch,
};

pub struct DocumentExpert<'s> {
    schema: &'s DataSchema,
}

impl<'s> DocumentExpert<'s> {
    pub fn new(schema: &'s DataSchema) -> DocumentExpert<'s> {
        DocumentExpert { schema }
    }

    pub fn extract_refs(&self, document_type: &DocumentType, data: &DocumentData) -> Result<Refs> {
        let mut refs = Refs::default();

        for field in self.schema.iter_fields(document_type)? {
            if let Some(value) = data.get(field.name) {
                refs.documents.extend(field.extract_refs(value));
                refs.collection.extend(field.extract_collection_refs(value));
                refs.blobs.extend(field.extract_blob_ids(value));
            }
        }

        Ok(refs)
    }

    pub fn pick_title_field(&self, document_type: &DocumentType) -> Result<Option<&Field>> {
        let field = self
            .schema
            .iter_fields(document_type)?
            .find(|field| field.could_be_title());

        Ok(field)
    }

    pub fn get_title(&self, document_type: &DocumentType, data: &DocumentData) -> Result<String> {
        let title_field = if let Some(title_field) = self.pick_title_field(document_type)? {
            title_field
        } else {
            return Ok(format!("Untitled {document_type}"));
        };

        data.get_str(title_field.name)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("title field {} is missing", title_field.name))
    }

    pub fn is_editable(&self, document_type: &DocumentType) -> Result<bool> {
        let is_editable = self
            .schema
            .iter_fields(document_type)?
            .any(|field| !field.readonly);

        Ok(is_editable)
    }

    pub fn search(
        &self,
        document_type: &DocumentType,
        data: &DocumentData,
        pattern: &str,
    ) -> Result<usize> {
        let title_field = self.pick_title_field(document_type)?;

        let mut final_score = 0;
        let multi_search = MultiSearch::new(pattern);

        for field in self.schema.iter_fields(document_type)? {
            let value = if let Some(value) = data.get(field.name) {
                value
            } else {
                continue;
            };

            let search_data = if let Some(search_data) = field.extract_search_data(value)? {
                search_data
            } else {
                continue;
            };

            let mut score = multi_search.search(&search_data);

            // increase score if field is a title
            if title_field.map_or(false, |title_field| title_field == field) {
                score *= 3;
            }

            final_score += score;
        }

        Ok(final_score)
    }
}
