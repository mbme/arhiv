use baza::schema::*;

pub const NOTE_TYPE: &str = "note";

pub fn get_note_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: NOTE_TYPE,
        title_format: "{title}",
        fields: vec![
            Field {
                name: "title",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
                for_subtypes: None,
            },
            Field {
                name: "data",
                field_type: FieldType::MarkupString {},
                mandatory: false,
                readonly: false,
                for_subtypes: None,
            },
        ],
        subtypes: None,
    }]
}
