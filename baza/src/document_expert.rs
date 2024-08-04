use anyhow::{anyhow, bail, Context, Result};
use tinytemplate::{format_unescaped, TinyTemplate};

use rs_utils::{is_http_url, is_image_path};

use crate::{
    entities::{Document, DocumentData, DocumentType, Id, Refs},
    schema::{download_attachment, Attachment, DataSchema, Field, FieldType, ATTACHMENT_TYPE},
    search::MultiSearch,
    BazaConnection,
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

    pub fn get_cover_attachment_id(&self, document: &Document) -> Result<Option<Id>> {
        if document.document_type.is(ATTACHMENT_TYPE) {
            let attachment: Attachment = document.clone().convert()?;

            if attachment.data.is_image() {
                return Ok(Some(attachment.id));
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

    pub async fn prepare_attachments(
        &self,
        document: &mut Document,
        tx: &mut BazaConnection,
    ) -> Result<()> {
        let fields = self
            .schema
            .iter_fields(&document.document_type)?
            .filter(|field| field.could_ref_attachments());

        for field in fields {
            match field.field_type {
                FieldType::Ref(_) => {
                    let value = document.data.get_str(field.name);
                    if let Some(value) = value {
                        if !is_http_url(value) {
                            continue;
                        }

                        if !is_image_path(value) {
                            bail!("Only image attachment URLs are supported, got '{value}'");
                        }

                        let attachment = download_attachment(value, tx).await?;

                        document.data.set(field.name, attachment.id);
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
                        if !is_http_url(value) {
                            continue;
                        }

                        if !is_image_path(value.clone()) {
                            bail!("Only image attachment URLs are supported, got '{value}'");
                        }

                        let attachment = download_attachment(value, tx).await?;

                        *value = attachment.id.to_string();
                    }

                    document.data.set(field.name, values);
                }
                _ => {
                    unreachable!("only ref fields might reference attachments");
                }
            }
        }

        Ok(())
    }
}
