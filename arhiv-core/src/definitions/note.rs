use crate::schema::*;

pub fn get_note_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: "note",
        is_internal: false,
        collection_of: Collection::None,
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
    }]
}
