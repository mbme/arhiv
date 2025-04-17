use baza::schema::*;

use super::fields::*;

pub const CONTACT_TYPE: &str = "contact";
pub const CONTACT_COLLECTION_TYPE: &str = "contact collection";

#[allow(clippy::too_many_lines)]
pub fn get_contact_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: CONTACT_TYPE,
            title_format: "${name}",
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(&[ASSET_TYPE]),
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "is_company",
                    field_type: FieldType::Flag {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "date_of_birth",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "date_of_death",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "primary_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "addresses",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "contacts",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "info",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
        DataDescription {
            document_type: CONTACT_COLLECTION_TYPE,
            title_format: "${name}",
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "contacts",
                    field_type: FieldType::RefList(&[CONTACT_TYPE]),
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
    ]
}
