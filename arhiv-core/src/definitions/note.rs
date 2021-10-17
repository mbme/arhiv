use crate::schema::*;

pub const NOTE_TYPE: &str = "note";

pub fn get_note_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: NOTE_TYPE,
        is_internal: false,
        collection_of: Collection::None,
        fields: vec![
            //
            Field {
                name: "title",
                field_type: FieldType::String {},
                mandatory: true,
            },
            Field {
                name: "data",
                field_type: FieldType::MarkupString {},
                mandatory: false,
            },
        ],
    }]
}
