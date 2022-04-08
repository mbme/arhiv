use super::fields::*;
use super::ATTACHMENT_TYPE;
use crate::schema::*;

pub const CONTACT_TYPE: &str = "contact";
pub const CONTACT_COLLECTION_TYPE: &str = "contact collection";

#[allow(clippy::too_many_lines)]
pub fn get_contact_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: CONTACT_TYPE,
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "is_company",
                    field_type: FieldType::Flag {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "date_of_birth",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "date_of_death",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "primary_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList(CONTACT_COLLECTION_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "addresses",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "contacts",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "info",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
            ],
            subtypes: &[],
        },
        DataDescription {
            document_type: CONTACT_COLLECTION_TYPE,
            collection_of: Collection::Type {
                document_type: CONTACT_TYPE,
                field: "collections",
            },
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
            ],
            subtypes: &[],
        },
    ]
}
