use anyhow::{anyhow, Context, Result};

use crate::{
    entities::{Document, DocumentClass, DocumentData, Refs},
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

    pub fn extract_refs(&self, document_type: &DocumentClass, data: &DocumentData) -> Result<Refs> {
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

    pub fn pick_title_field(&self, document_type: &DocumentClass) -> Result<Option<&Field>> {
        let field = self
            .schema
            .iter_fields(document_type)?
            .find(|field| field.could_be_title());

        Ok(field)
    }

    pub fn get_title(&self, document_type: &DocumentClass, data: &DocumentData) -> Result<String> {
        let title_field = if let Some(title_field) = self.pick_title_field(document_type)? {
            title_field
        } else {
            return Ok(format!("Untitled {document_type}"));
        };

        data.get_str(title_field.name)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("title field {} is missing", title_field.name))
    }

    pub fn is_editable(&self, document_type: &DocumentClass) -> Result<bool> {
        let is_editable = self
            .schema
            .iter_fields(document_type)?
            .any(|field| !field.readonly);

        Ok(is_editable)
    }

    pub fn search(
        &self,
        document_type: &DocumentClass,
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

    fn find_collection_field_for(
        &self,
        collection_type: &DocumentClass,
        document_type: &DocumentClass,
    ) -> Result<&Field> {
        self.schema
            .iter_fields(collection_type)?
            .find(|field| field.can_collect(document_type))
            .context(anyhow!(
                "document {collection_type} can't collect {document_type}",
            ))
    }

    pub fn add_document_to_collection(
        &self,
        document: &Document,
        collection: &mut Document,
    ) -> Result<()> {
        let field = self.find_collection_field_for(&collection.class, &document.class)?;

        collection.data.add_to_ref_list(field.name, &document.id)?;

        Ok(())
    }

    pub fn remove_document_from_collection(
        &self,
        document: &Document,
        collection: &mut Document,
    ) -> Result<()> {
        let field = self.find_collection_field_for(&collection.class, &document.class)?;

        collection
            .data
            .remove_from_ref_list(field.name, &document.id)?;

        Ok(())
    }

    pub fn reorder_refs(
        &self,
        collection: &mut Document,
        document: &Document,
        new_pos: usize,
    ) -> Result<()> {
        let field = self.find_collection_field_for(&collection.class, &document.class)?;

        let mut ref_list = collection.data.get_ref_list(field.name)?.context(format!(
            "collection {} field {} is empty",
            collection.id, field.name
        ))?;

        let pos = ref_list
            .iter()
            .position(|id| id == &document.id)
            .context(format!(
                "collection {} field {} doesn't include document {}",
                collection.id, field.name, document.id
            ))?;

        if pos == new_pos {
            return Ok(());
        }

        let ref_to_move = ref_list.remove(pos);
        ref_list.insert(new_pos, ref_to_move);

        collection.data.set(field.name, ref_list);

        Ok(())
    }
}
