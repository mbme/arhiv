use serde_json::json;

use crate::{
    schema::{DataDescription, DataSchema, Field, FieldType},
    Baza,
};

use super::new_document;

#[test]
fn test_validation_mandatory() {
    let baza = Baza::new_with_schema(DataSchema::new(
        "test",
        vec![DataDescription {
            document_type: "test_type",
            title_format: "title",
            fields: vec![Field {
                name: "test",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
                for_subtypes: None,
            }],
            subtypes: None,
        }],
    ));

    {
        let mut tx = baza.get_tx().unwrap();
        let mut document = new_document(json!({}));
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_err());
    }

    {
        let mut tx = baza.get_tx().unwrap();
        let mut document = new_document(json!({ "test": "test" }));
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_ok());
    }
}

#[test]
fn test_validation_readonly() {
    let baza = Baza::new_with_schema(DataSchema::new(
        "test",
        vec![DataDescription {
            document_type: "test_type",
            title_format: "title",
            fields: vec![Field {
                name: "test",
                field_type: FieldType::String {},
                mandatory: false,
                readonly: true,
                for_subtypes: None,
            }],
            subtypes: None,
        }],
    ));

    {
        let mut tx = baza.get_tx().unwrap();

        let mut document = new_document(json!({ "test": "test" }));
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_ok());

        document.data = json!({ "test": None::<String> }).try_into().unwrap();
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_err());
    }

    {
        let mut tx = baza.get_tx().unwrap();

        let mut document = new_document(json!({}));
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_ok());

        document.data = json!({ "test": "test" }).try_into().unwrap();
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_err());

        document.data = json!({ "test": None::<String> }).try_into().unwrap();
        let result = tx.stage_document(&mut document, None);
        assert!(result.is_ok());
    }
}
