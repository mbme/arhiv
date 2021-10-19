use super::fields::*;
use crate::{entities::ATTACHMENT_TYPE, schema::*};

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
                },
                Field {
                    name: "authors",
                    field_type: FieldType::People {},
                    mandatory: true,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: false,
                },
                Field {
                    name: "language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                },
                Field {
                    name: "original_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                },
                Field {
                    name: "publication_date",
                    field_type: FieldType::Date {},
                    mandatory: false,
                },
                Field {
                    name: "translators",
                    field_type: FieldType::People {},
                    mandatory: false,
                },
                Field {
                    name: "publisher",
                    field_type: FieldType::String {},
                    mandatory: false,
                },
                Field {
                    name: "pages",
                    field_type: FieldType::NaturalNumber {},
                    mandatory: false,
                },
                Field {
                    name: "ISBN",
                    field_type: FieldType::ISBN {},
                    mandatory: false,
                },
                // for audiobooks
                Field {
                    name: "narrators",
                    field_type: FieldType::People {},
                    mandatory: false,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    mandatory: false,
                },
                // --
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList(BOOK_COLLECTION_TYPE),
                    mandatory: false,
                },
                Field {
                    name: "completed",
                    field_type: FieldType::Flag {},
                    mandatory: false,
                },
                Field {
                    name: "rating",
                    field_type: RATING_FIELD,
                    mandatory: false,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
            ],
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
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
            ],
        },
    ]
}
