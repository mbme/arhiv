use baza::schema::*;

pub const PROJECT_TYPE: &str = "project";

pub const TASK_TYPE: &str = "task";

pub const TASK_STATUS: &[&str] = &["Todo", "InProgress", "Done", "Cancelled"];

pub fn get_task_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: PROJECT_TYPE,
            fields: vec![
                //
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
                    name: "tasks",
                    field_type: FieldType::RefList(TASK_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                },
            ],
            subtypes: None,
        },
        DataDescription {
            document_type: TASK_TYPE,
            fields: vec![
                Field {
                    name: "title",
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
                    name: "status",
                    field_type: FieldType::Enum(TASK_STATUS),
                    mandatory: true,
                    readonly: false,
                    for_subtypes: None,
                },
            ],
            subtypes: None,
        },
    ]
}
