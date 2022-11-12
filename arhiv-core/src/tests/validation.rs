use serde_json::json;

use crate::test_arhiv::TestArhiv;
use baza::schema::{Collection, DataDescription, DataSchema, Field, FieldType};

use super::utils::*;

#[test]
fn test_validation_mandatory() {
    let arhiv = TestArhiv::new_prime_with_schema(DataSchema::new(vec![DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![Field {
            name: "test",
            field_type: FieldType::String {},
            mandatory: true,
            readonly: false,
            for_subtypes: None,
        }],
        subtypes: None,
    }]));

    {
        let tx = arhiv.baza.get_tx().unwrap();
        let mut document = new_document(json!({}));
        let result = tx.stage_document(&mut document);
        assert!(result.is_err());
    }

    {
        let tx = arhiv.baza.get_tx().unwrap();
        let mut document = new_document(json!({ "test": "test" }));
        let result = tx.stage_document(&mut document);
        assert!(result.is_ok());
    }
}

#[test]
fn test_validation_readonly() {
    let arhiv = TestArhiv::new_prime_with_schema(DataSchema::new(vec![DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![Field {
            name: "test",
            field_type: FieldType::String {},
            mandatory: false,
            readonly: true,
            for_subtypes: None,
        }],
        subtypes: None,
    }]));

    {
        let tx = arhiv.baza.get_tx().unwrap();

        let mut document = new_document(json!({ "test": "test" }));
        let result = tx.stage_document(&mut document);
        assert!(result.is_ok());

        document.data = json!({ "test": None::<String> }).try_into().unwrap();
        let result = tx.stage_document(&mut document);
        assert!(result.is_err());
    }

    {
        let tx = arhiv.baza.get_tx().unwrap();

        let mut document = new_document(json!({}));
        let result = tx.stage_document(&mut document);
        assert!(result.is_ok());

        document.data = json!({ "test": "test" }).try_into().unwrap();
        let result = tx.stage_document(&mut document);
        assert!(result.is_err());

        document.data = json!({ "test": None::<String> }).try_into().unwrap();
        let result = tx.stage_document(&mut document);
        assert!(result.is_ok());
    }
}
