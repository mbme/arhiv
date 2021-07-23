use super::fields::*;
use crate::{entities::ATTACHMENT_TYPE, schema::*};

pub fn get_book_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: "book",
            is_internal: false,
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    optional: false,
                },
                Field {
                    name: "authors",
                    field_type: FieldType::People {},
                    optional: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    optional: true,
                },
                Field {
                    name: "language",
                    field_type: language_field(),
                    optional: true,
                },
                Field {
                    name: "original_language",
                    field_type: language_field(),
                    optional: true,
                },
                Field {
                    name: "publication_date",
                    field_type: FieldType::Date {},
                    optional: true,
                },
                Field {
                    name: "translators",
                    field_type: FieldType::People {},
                    optional: true,
                },
                Field {
                    name: "publisher",
                    field_type: FieldType::String {},
                    optional: true,
                },
                Field {
                    name: "pages",
                    field_type: FieldType::NaturalNumber {},
                    optional: true,
                },
                Field {
                    name: "ISBN",
                    field_type: FieldType::ISBN {},
                    optional: true,
                },
                // for audiobooks
                Field {
                    name: "narrators",
                    field_type: FieldType::People {},
                    optional: true,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    optional: true,
                },
                // --
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList("book collection"),
                    optional: true,
                },
                Field {
                    name: "completed",
                    field_type: FieldType::Flag {},
                    optional: true,
                },
                Field {
                    name: "rating",
                    field_type: rating_field(),
                    optional: true,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
            ],
        },
        DataDescription {
            document_type: "book collection",
            is_internal: false,
            collection_of: Collection::Type {
                document_type: "book",
                field: "collections",
            },
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    optional: false,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
            ],
        },
    ]
}
