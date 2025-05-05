use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, bail};

use rs_utils::{is_http_url, is_image_url, parse_url, render_template_with_vars, value_as_string};

use crate::{
    BazaManager,
    entities::{Document, DocumentData, DocumentType, Id, Refs},
    schema::{ASSET_TYPE, Asset, DataSchema, Field, FieldType, download_asset},
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
            }
        }

        Ok(refs)
    }

    pub fn get_title(&self, document_type: &DocumentType, data: &DocumentData) -> Result<String> {
        let mut title_fields = HashMap::new();
        for field in self.schema.iter_fields(document_type)? {
            if field.could_be_in_title() {
                title_fields.insert(field.name, value_as_string(data.get(field.name)));
            }
        }

        render_template_with_vars(
            self.schema
                .get_data_description(document_type)?
                .title_format,
            &title_fields,
        )
        .map_err(|err| anyhow!("failed to render title for {document_type}: {err}"))
    }

    fn pick_cover_field(&self, document_type: &DocumentType) -> Result<Option<&Field>> {
        let field = self
            .schema
            .iter_fields(document_type)?
            .find(|field| field.could_be_cover());

        Ok(field)
    }

    pub fn get_cover_asset_id(&self, document: &Document) -> Result<Option<Id>> {
        if document.document_type.is(ASSET_TYPE) {
            let asset: Asset = document.clone().convert()?;

            if asset.data.is_image() {
                return Ok(Some(asset.id));
            }
        }

        let cover_field =
            if let Some(cover_field) = self.pick_cover_field(&document.document_type)? {
                cover_field
            } else {
                return Ok(None);
            };

        Ok(document.data.get_str(cover_field.name).map(From::from))
    }

    pub fn is_editable(&self, document_type: &DocumentType) -> Result<bool> {
        let is_editable = self
            .schema
            .iter_fields(document_type)?
            .any(|field| !field.readonly);

        Ok(is_editable)
    }

    fn find_collection_field_for(
        &self,
        collection_type: &DocumentType,
        document_type: &DocumentType,
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
        let field =
            self.find_collection_field_for(&collection.document_type, &document.document_type)?;

        collection.data.add_to_ref_list(field.name, &document.id)?;

        Ok(())
    }

    pub fn remove_document_from_collection(
        &self,
        document: &Document,
        collection: &mut Document,
    ) -> Result<()> {
        let field =
            self.find_collection_field_for(&collection.document_type, &document.document_type)?;

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
        let field =
            self.find_collection_field_for(&collection.document_type, &document.document_type)?;

        let mut ref_list = collection
            .data
            .get_ref_list(field.name)?
            .context(format!(
                "collection {} field {} is empty",
                collection.id, field.name
            ))?
            .into_iter()
            .map(Id::from)
            .collect::<Vec<_>>();

        let pos = ref_list
            .iter()
            .position(|id| *id == document.id)
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

    pub async fn prepare_assets(
        &self,
        document: &mut Document,
        baza_manager: &BazaManager,
    ) -> Result<()> {
        let fields = self
            .schema
            .iter_fields(&document.document_type)?
            .filter(|field| field.could_ref_assets());

        for field in fields {
            match field.field_type {
                FieldType::Ref(_) => {
                    let value = document.data.get_str(field.name);
                    if let Some(value) = value {
                        let url = if let Ok(url) = parse_url(value) {
                            url
                        } else {
                            continue;
                        };

                        if !is_http_url(&url) {
                            continue;
                        }

                        if !is_image_url(&url) {
                            bail!("Only image asset URLs are supported, got '{value}'");
                        }

                        let asset = download_asset(value, baza_manager).await?;

                        document.data.set(field.name, asset.id);
                    }
                }

                FieldType::RefList(_) => {
                    let mut values = document
                        .data
                        .get_ref_list(field.name)?
                        .unwrap_or_default()
                        .into_iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>();

                    for value in values.iter_mut() {
                        let url = if let Ok(url) = parse_url(value) {
                            url
                        } else {
                            continue;
                        };

                        if !is_http_url(&url) {
                            continue;
                        }

                        if !is_image_url(&url) {
                            bail!("Only image asset URLs are supported, got '{value}'");
                        }

                        let asset = download_asset(value, baza_manager).await?;

                        *value = asset.id.to_string();
                    }

                    document.data.set(field.name, values);
                }
                _ => {
                    unreachable!("only ref fields might reference assets");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entities::{DocumentData, DocumentType},
        schema::DataSchema,
    };

    use super::DocumentExpert;

    #[test]
    fn test_title() {
        let schema = DataSchema::new_test_schema();
        let expert = DocumentExpert::new(&schema);

        let mut data = DocumentData::new();
        data.set("test", "test");
        let title = expert
            .get_title(&DocumentType::new("test_type"), &data)
            .unwrap();
        assert_eq!(title, "test");
    }
}
