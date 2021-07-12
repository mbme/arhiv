use crate::schema::*;

pub fn get_task_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: "project",
            is_internal: false,
            collection_of: Collection::Type("task"),
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
            collection_of: Collection::None,
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
    ]
}
