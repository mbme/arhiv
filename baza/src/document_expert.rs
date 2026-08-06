use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, ensure};

use baza_common::{render_template_with_vars, value_as_string};

use crate::{
    entities::{Document, DocumentData, DocumentType, Id, Refs},
    schema::{ASSET_TYPE, Asset, DataSchema, Field, FieldType},
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

    /// Returns ordered member ids from every collection field on the document.
    pub fn collection_member_ids(&self, collection: &Document) -> Result<Vec<Id>> {
        let mut ids = Vec::new();
        let mut has_collection_field = false;

        for field in self.schema.iter_fields(&collection.document_type)? {
            if !matches!(&field.field_type, FieldType::RefList(_)) {
                continue;
            }

            has_collection_field = true;

            let Some(field_ids) = collection.data.get_ref_list(field.name)? else {
                continue;
            };

            ids.extend(field_ids.into_iter().map(Id::from));
        }

        ensure!(
            has_collection_field,
            "document {} of type {} isn't a collection",
            collection.id,
            collection.document_type
        );

        Ok(ids)
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
        ensure!(
            !document.is_erased(),
            "erased document {} can't be added to collection {}",
            document.id,
            collection.id
        );

        let field =
            self.find_collection_field_for(&collection.document_type, &document.document_type)?;

        collection.data.add_to_ref_list(field.name, &document.id)?;

        Ok(())
    }

    fn find_collection_field_containing_member(
        &self,
        collection: &Document,
        id: &Id,
    ) -> Result<&'static str> {
        let mut fields = Vec::new();
        let mut has_collection_field = false;

        for field in self.schema.iter_fields(&collection.document_type)? {
            if !matches!(&field.field_type, FieldType::RefList(_)) {
                continue;
            }

            has_collection_field = true;

            let Some(field_ids) = collection.data.get_ref_list(field.name)? else {
                continue;
            };

            if field_ids
                .into_iter()
                .any(|field_id| field_id == id.as_ref())
            {
                fields.push(field.name);
            }
        }

        ensure!(
            has_collection_field,
            "document {} of type {} isn't a collection",
            collection.id,
            collection.document_type
        );
        ensure!(
            !fields.is_empty(),
            "collection {} doesn't include document {}",
            collection.id,
            id
        );
        ensure!(
            fields.len() == 1,
            "collection {} includes document {} in multiple fields",
            collection.id,
            id
        );

        Ok(fields[0])
    }

    /// Removes an existing collection member by id, independent of the member's current type.
    pub fn remove_member_from_collection(&self, collection: &mut Document, id: &Id) -> Result<()> {
        let field_name = self.find_collection_field_containing_member(collection, id)?;

        collection.data.remove_from_ref_list(field_name, id)?;

        Ok(())
    }

    /// Reorders an existing collection member by id within its current collection field.
    pub fn reorder_collection_member(
        &self,
        collection: &mut Document,
        id: &Id,
        new_pos: usize,
    ) -> Result<()> {
        let field_name = self.find_collection_field_containing_member(collection, id)?;
        let mut ref_list = collection
            .data
            .get_ref_list(field_name)?
            .context(format!(
                "collection {} field {} is empty",
                collection.id, field_name
            ))?
            .into_iter()
            .map(Id::from)
            .collect::<Vec<_>>();

        let pos = ref_list
            .iter()
            .position(|item| item == id)
            .context(format!(
                "collection {} field {} doesn't include document {}",
                collection.id, field_name, id
            ))?;

        ensure!(
            new_pos < ref_list.len(),
            "new position {new_pos} is out of bounds for collection {} field {} with {} members",
            collection.id,
            field_name,
            ref_list.len()
        );

        if pos == new_pos {
            return Ok(());
        }

        let ref_to_move = ref_list.remove(pos);
        ref_list.insert(new_pos, ref_to_move);

        collection.data.set(field_name, ref_list);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entities::{Document, DocumentData, DocumentType},
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

    #[test]
    fn reorder_collection_member_rejects_out_of_bounds_position() {
        let schema = DataSchema::new(
            "test",
            vec![
                DataDescription {
                    document_type: "collection",
                    title_format: "${title}",
                    fields: vec![Field {
                        name: "members",
                        field_type: FieldType::RefList(&["member"]),
                        mandatory: false,
                        readonly: false,
                    }],
                },
                DataDescription {
                    document_type: "member",
                    title_format: "${title}",
                    fields: vec![],
                },
            ],
        );
        let expert = DocumentExpert::new(&schema);
        let member = Document::new_with_data(DocumentType::new("member"), DocumentData::new());
        let mut collection =
            Document::new_with_data(DocumentType::new("collection"), DocumentData::new());
        collection.data.set("members", vec![member.id.clone()]);

        let err = expert
            .reorder_collection_member(&mut collection, &member.id, 1)
            .unwrap_err();

        assert!(err.to_string().contains("out of bounds"));
    }

    #[test]
    fn remove_member_from_collection_uses_existing_membership_field() {
        let schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "collection",
                title_format: "${title}",
                fields: vec![Field {
                    name: "members",
                    field_type: FieldType::RefList(&["member"]),
                    mandatory: false,
                    readonly: false,
                }],
            }],
        );
        let expert = DocumentExpert::new(&schema);
        let member_id = crate::entities::Id::new();
        let mut collection =
            Document::new_with_data(DocumentType::new("collection"), DocumentData::new());
        collection.data.set("members", vec![member_id.clone()]);

        expert
            .remove_member_from_collection(&mut collection, &member_id)
            .unwrap();

        assert_eq!(
            collection.data.get_ref_list("members").unwrap().unwrap(),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn add_document_to_collection_rejects_erased_document() {
        let schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "collection",
                title_format: "${title}",
                fields: vec![Field {
                    name: "members",
                    field_type: FieldType::RefList(&[]),
                    mandatory: false,
                    readonly: false,
                }],
            }],
        );
        let expert = DocumentExpert::new(&schema);
        let mut document =
            Document::new_with_data(DocumentType::new("member"), DocumentData::new());
        document.erase();
        let mut collection =
            Document::new_with_data(DocumentType::new("collection"), DocumentData::new());

        let err = expert
            .add_document_to_collection(&document, &mut collection)
            .unwrap_err();

        assert!(err.to_string().contains("erased document"));
    }

    #[test]
    fn collection_member_ids_preserves_field_and_member_order() {
        let schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "collection",
                title_format: "${title}",
                fields: vec![
                    Field {
                        name: "primary",
                        field_type: FieldType::RefList(&["member"]),
                        mandatory: false,
                        readonly: false,
                    },
                    Field {
                        name: "secondary",
                        field_type: FieldType::RefList(&["member"]),
                        mandatory: false,
                        readonly: false,
                    },
                ],
            }],
        );
        let expert = DocumentExpert::new(&schema);
        let first = Document::new_with_data(DocumentType::new("member"), DocumentData::new());
        let second = Document::new_with_data(DocumentType::new("member"), DocumentData::new());
        let third = Document::new_with_data(DocumentType::new("member"), DocumentData::new());
        let mut collection =
            Document::new_with_data(DocumentType::new("collection"), DocumentData::new());
        collection
            .data
            .set("primary", vec![first.id.clone(), second.id.clone()]);
        collection.data.set("secondary", vec![third.id.clone()]);

        let member_ids = expert.collection_member_ids(&collection).unwrap();

        assert_eq!(member_ids, [first.id, second.id, third.id]);
    }

    #[test]
    fn collection_member_ids_rejects_document_without_collection_fields() {
        let schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "note",
                title_format: "${title}",
                fields: vec![],
            }],
        );
        let expert = DocumentExpert::new(&schema);
        let document = Document::new_with_data(DocumentType::new("note"), DocumentData::new());

        let err = expert.collection_member_ids(&document).unwrap_err();

        assert!(err.to_string().contains("isn't a collection"));
    }
}
