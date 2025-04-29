use baza::schema::*;

pub const TAG_TYPE: &str = "tag";

pub fn get_tag_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: TAG_TYPE,
        title_format: "${title}",
        fields: vec![
            Field {
                name: "title",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
            },
            Field {
                name: "items",
                field_type: FieldType::RefList(&[]),
                mandatory: false,
                readonly: false,
            },
        ],
    }]
}
