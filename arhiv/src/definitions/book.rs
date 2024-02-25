use baza::schema::*;

use super::fields::*;

pub const BOOK_TYPE: &str = "book";
pub const BOOK_COLLECTION_TYPE: &str = "book collection";

#[allow(clippy::too_many_lines)]
pub fn get_book_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: BOOK_TYPE,
            title_format: "{authors} - {title}",
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "authors",
                    field_type: FieldType::People {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "original_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "publication_date",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "translators",
                    field_type: FieldType::People {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "publisher",
                    field_type: FieldType::String {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "pages",
                    field_type: FieldType::NaturalNumber {},
                    mandatory: false,
                    readonly: false,
                },
                // for audiobooks
                Field {
                    name: "narrators",
                    field_type: FieldType::People {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    mandatory: false,
                    readonly: false,
                },
                // --
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "status",
                    field_type: STATUS_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "rating",
                    field_type: RATING_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
        DataDescription {
            document_type: BOOK_COLLECTION_TYPE,
            title_format: "{name}",
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
                    name: "books",
                    field_type: FieldType::RefList(BOOK_TYPE),
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
    ]
}
