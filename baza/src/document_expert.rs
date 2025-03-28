use anyhow::{anyhow, bail, Context, Result};
use tinytemplate::{format_unescaped, TinyTemplate};

use rs_utils::{is_http_url, is_image_url, parse_url};

use crate::{
    baza2::BazaManager,
    entities::{Document, DocumentData, DocumentType, Id, Refs},
    schema::{download_asset, Asset, DataSchema, Field, FieldType, ASSET_TYPE},
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
            }
        }

        Ok(refs)
    }

    pub fn get_title(&self, document_type: &DocumentType, data: &DocumentData) -> Result<String> {
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&format_unescaped);

        tt.add_template(
            "title",
            self.schema
                .get_data_description(document_type)?
                .title_format,
        )
        .context(anyhow!(
            "failed to compile title template for {document_type}"
        ))?;

        tt.render("title", data)
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

    pub fn search(
        &self,
        document_type: &DocumentType,
        data: &DocumentData,
        pattern: &str,
    ) -> Result<usize> {
        let title = self.get_title(document_type, data)?;

        let mut final_score = 0;
        let multi_search = MultiSearch::new(pattern);

        // increase score if field is a title
        let title_score = multi_search.search(&title) * 3;
        final_score += title_score;

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

            final_score += multi_search.search(&search_data);
        }

        Ok(final_score)
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
