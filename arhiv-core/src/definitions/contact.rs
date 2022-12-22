use baza::schema::*;

use super::fields::*;
use super::ATTACHMENT_TYPE;

pub const CONTACT_TYPE: &str = "contact";
pub const CONTACT_COLLECTION_TYPE: &str = "contact collection";

#[allow(clippy::too_many_lines)]
pub fn get_contact_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: CONTACT_TYPE,
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "is_company",
                    field_type: FieldType::Flag {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "date_of_birth",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "date_of_death",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "primary_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList(CONTACT_COLLECTION_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "addresses",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "contacts",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "info",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
            ],
            subtypes: None,
        },
        DataDescription {
            document_type: CONTACT_COLLECTION_TYPE,
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "contacts",
                    field_type: FieldType::RefList(CONTACT_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
            ],
            subtypes: None,
        },
    ]
}
