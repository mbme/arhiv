use baza::schema::*;

use super::fields::*;
use super::ATTACHMENT_TYPE;

pub const BOOK_TYPE: &str = "book";
pub const BOOK_COLLECTION_TYPE: &str = "book collection";

#[allow(clippy::too_many_lines)]
pub fn get_book_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: BOOK_TYPE,
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "authors",
                    field_type: FieldType::People {},
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
                    name: "language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "original_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "publication_date",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "translators",
                    field_type: FieldType::People {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "publisher",
                    field_type: FieldType::String {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "pages",
                    field_type: FieldType::NaturalNumber {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                // for audiobooks
                Field {
                    name: "narrators",
                    field_type: FieldType::People {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                // --
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList(BOOK_COLLECTION_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "status",
                    field_type: STATUS_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "rating",
                    field_type: RATING_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
            ],
            subtypes: None,
        },
        DataDescription {
            document_type: BOOK_COLLECTION_TYPE,
            collection_of: Collection::Type {
                document_type: BOOK_TYPE,
                field: "collections",
            },
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
            ],
            subtypes: None,
        },
    ]
}
