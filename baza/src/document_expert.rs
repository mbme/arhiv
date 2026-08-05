use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};

use baza_common::{render_template_with_vars, value_as_string};

use crate::{
    entities::{Document, DocumentData, DocumentType, Id, Refs},
    schema::{ASSET_TYPE, Asset, DataSchema, Field},
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

    /// Returns schema fields whose values may be materialized into asset document refs.
    ///
    /// This pure schema query lets callers classify each field value as an id, a
    /// local reference, or a remote URL handled by application-layer IO.
    pub fn asset_ref_fields(&self, document_type: &DocumentType) -> Result<Vec<&Field>> {
        let fields = self
            .schema
            .iter_fields(document_type)?
            .filter(|field| field.could_ref_assets())
            .collect();

        Ok(fields)
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
}

#[cfg(test)]
mod tests {
    use crate::{
        entities::{DocumentData, DocumentType},
        schema::{ASSET_TYPE, DataDescription, DataSchema, Field, FieldType},
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

    #[test]
    fn asset_ref_fields_returns_only_fields_that_can_reference_assets() {
        let schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "test_type",
                title_format: "${title}",
                fields: vec![
                    Field {
                        name: "title",
                        field_type: FieldType::String {},
                        mandatory: false,
                        readonly: false,
                    },
                    Field {
                        name: "cover",
                        field_type: FieldType::Ref(&[ASSET_TYPE]),
                        mandatory: false,
                        readonly: false,
                    },
                    Field {
                        name: "related",
                        field_type: FieldType::Ref(&["note"]),
                        mandatory: false,
                        readonly: false,
                    },
                    Field {
                        name: "gallery",
                        field_type: FieldType::RefList(&[ASSET_TYPE]),
                        mandatory: false,
                        readonly: false,
                    },
                    Field {
                        name: "mixed_refs",
                        field_type: FieldType::RefList(&["note", ASSET_TYPE]),
                        mandatory: false,
                        readonly: false,
                    },
                ],
            }],
        );
        let expert = DocumentExpert::new(&schema);

        let fields = expert
            .asset_ref_fields(&DocumentType::new("test_type"))
            .unwrap();
        let field_names = fields.iter().map(|field| field.name).collect::<Vec<_>>();

        assert_eq!(field_names, ["cover", "gallery", "mixed_refs"]);
    }
}
