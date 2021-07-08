use crate::{entities::ATTACHMENT_TYPE, schema::*};

pub fn get_data_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: "note",
            is_internal: false,
            collection_of: None,
            fields: vec![
                //
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    optional: false,
                },
                Field {
                    name: "data",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
            ],
        },
        DataDescription {
            document_type: "project",
            is_internal: false,
            collection_of: Some(Collection { item_type: "task" }),
            fields: vec![
                //
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
        DataDescription {
            document_type: "task",
            is_internal: false,
            collection_of: None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    optional: false,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
                Field {
                    name: "status",
                    field_type: FieldType::Enum(vec![
                        "Inbox",
                        "InProgress",
                        "Paused",
                        "Todo",
                        "Later",
                        "Done",
                        "Cancelled",
                    ]),
                    optional: false,
                },
                Field {
                    name: "project",
                    field_type: FieldType::Ref("project"),
                    optional: false,
                },
            ],
        },
        DataDescription {
            document_type: "book",
            is_internal: false,
            collection_of: None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    optional: false,
                },
                Field {
                    name: "authors",
                    field_type: FieldType::String {},
                    optional: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    optional: true,
                },
                Field {
                    name: "language",
                    field_type: FieldType::Enum(vec!["Ukrainian", "English", "Russian"]),
                    optional: true,
                },
                Field {
                    name: "translators",
                    field_type: FieldType::String {},
                    optional: true,
                },
                Field {
                    name: "publisher",
                    field_type: FieldType::String {},
                    optional: true,
                },
                Field {
                    name: "publication_date",
                    field_type: FieldType::Date {},
                    optional: true,
                },
                Field {
                    name: "pages",
                    field_type: FieldType::NaturalNumber {},
                    optional: true,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
                Field {
                    name: "ISBN",
                    field_type: FieldType::ISBN {},
                    optional: true,
                },
                Field {
                    name: "rating",
                    field_type: FieldType::Enum(vec![
                        "Very Bad", //
                        "Bad", "Average", "Fine", "Good", "Great",
                    ]),
                    optional: true,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
            ],
        },
    ]
}
