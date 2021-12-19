use anyhow::*;
use serde_json::json;

use crate::schema::*;

use super::utils::*;

#[test]
fn test_validation_mandatory() -> Result<()> {
    let arhiv = new_prime_with_schema(DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![Field {
            name: "test",
            field_type: FieldType::String {},
            mandatory: true,
            readonly: false,
        }],
    });

    let mut document = new_document(json!({}));
    let result = arhiv.stage_document(&mut document);
    assert!(result.is_err());

    let mut document = new_document(json!({ "test": "test" }));
    let result = arhiv.stage_document(&mut document);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_validation_readonly() -> Result<()> {
    let arhiv = new_prime_with_schema(DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![Field {
            name: "test",
            field_type: FieldType::String {},
            mandatory: false,
            readonly: true,
        }],
    });

    {
        let mut document = new_document(json!({ "test": "test" }));
        let result = arhiv.stage_document(&mut document);
        assert!(result.is_ok());

        document.data = json!({ "test": None::<String> }).try_into()?;
        let result = arhiv.stage_document(&mut document);
        assert!(result.is_err());
    }

    {
        let mut document = new_document(json!({}));
        let result = arhiv.stage_document(&mut document);
        assert!(result.is_ok());

        document.data = json!({ "test": "test" }).try_into()?;
        let result = arhiv.stage_document(&mut document);
        assert!(result.is_err());

        document.data = json!({ "test": None::<String> }).try_into()?;
        let result = arhiv.stage_document(&mut document);
        assert!(result.is_ok());
    }

    Ok(())
}
