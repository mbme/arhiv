use anyhow::Result;
use serde_json::json;

use super::utils::*;
use crate::{
    schema::{Collection, DataDescription, DataSchema, Field, FieldType},
    test_arhiv::TestArhiv,
    Filter, OrderBy,
};

#[test]
fn test_pagination() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    {
        let tx = arhiv.get_tx()?;

        tx.stage_document(&mut empty_document())?;
        tx.stage_document(&mut empty_document())?;

        tx.commit()?;
    }

    let page = arhiv.list_documents(Filter::default().page_size(1))?;

    assert_eq!(page.items.len(), 1);
    assert!(page.has_more);

    Ok(())
}

#[tokio::test]
async fn test_modes() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    // committed
    {
        let tx = arhiv.get_tx()?;
        tx.stage_document(&mut new_document(json!({ "test": "1" })))?;
        tx.commit()?;
    }

    arhiv.sync().await?;

    // staged
    {
        let tx = arhiv.get_tx()?;
        tx.stage_document(&mut new_document(json!({ "test": "3" })))?;
        tx.commit()?;
    }

    {
        // test default
        let page = arhiv.list_documents(Filter {
            order: vec![OrderBy::UpdatedAt { asc: false }],
            ..Filter::default()
        })?;

        assert_eq!(
            get_values(page),
            vec![json!({ "test": "3" }), json!({ "test": "1" }),]
        );
    }

    {
        // test staged
        let page = arhiv.list_documents(Filter::default().only_staged())?;

        assert_eq!(get_values(page), vec![json!({ "test": "3" })]);
    }

    Ok(())
}

#[test]
fn test_order_by_enum_field() -> Result<()> {
    let arhiv = TestArhiv::new_prime_with_schema(DataSchema::new(vec![DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![Field {
            name: "enum",
            field_type: FieldType::Enum(&["low", "high", "medium", "other"]),
            mandatory: false,
            readonly: false,
            for_subtypes: None,
        }],
        subtypes: None,
    }]));

    {
        let tx = arhiv.get_tx()?;

        tx.stage_document(&mut new_document(json!({ "enum": "low" })))?;
        tx.stage_document(&mut new_document(json!({ "enum": "high" })))?;
        tx.stage_document(&mut new_document(json!({ "enum": "other" })))?;
        tx.stage_document(&mut new_document(json!({ "enum": "medium" })))?;

        tx.commit()?;
    }

    let page = arhiv.list_documents(Filter {
        order: vec![OrderBy::EnumField {
            selector: "$.enum".to_string(),
            asc: true,
            enum_order: vec!["high".to_string(), "medium".to_string(), "low".to_string()],
        }],
        ..Filter::default()
    })?;

    assert_eq!(
        get_values(page),
        vec![
            json!({ "enum": "high" }),
            json!({ "enum": "medium" }),
            json!({ "enum": "low" }),
            json!({ "enum": "other" }),
        ]
    );

    Ok(())
}

#[test]
fn test_multiple_order_by() -> Result<()> {
    let arhiv = TestArhiv::new_prime_with_schema(DataSchema::new(vec![DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![
            Field {
                name: "prop",
                field_type: FieldType::String {},
                mandatory: false,
                readonly: false,
                for_subtypes: None,
            },
            Field {
                name: "other",
                field_type: FieldType::String {},
                mandatory: false,
                readonly: false,
                for_subtypes: None,
            },
        ],
        subtypes: None,
    }]));

    {
        let tx = arhiv.get_tx()?;

        tx.stage_document(&mut new_document(json!({ "prop": "b", "other": "2" })))?;
        tx.stage_document(&mut new_document(json!({ "prop": "a", "other": "1" })))?;
        tx.stage_document(&mut new_document(json!({ "prop": "a", "other": "2" })))?;
        tx.stage_document(&mut new_document(json!({ "prop": "b", "other": "1" })))?;

        tx.commit()?;
    }

    let page = arhiv.list_documents(Filter {
        order: vec![
            OrderBy::Field {
                selector: "$.prop".to_string(),
                asc: true,
            },
            OrderBy::Field {
                selector: "$.other".to_string(),
                asc: false,
            },
        ],
        ..Filter::default()
    })?;

    assert_eq!(
        get_values(page),
        vec![
            json!({ "prop": "a", "other": "2" }),
            json!({ "prop": "a", "other": "1" }),
            json!({ "prop": "b", "other": "2" }),
            json!({ "prop": "b", "other": "1" }),
        ],
    );

    Ok(())
}

#[tokio::test]
async fn test_matcher() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    {
        let tx = arhiv.get_tx()?;

        tx.stage_document(&mut new_document(json!({ "test": "value" })))?;
        tx.stage_document(&mut new_document(json!({ "test": "value1" })))?;

        tx.commit()?;
    }

    {
        // test unexpected type
        let page = arhiv.list_documents(Filter::default().with_document_type("random"))?;

        let empty: Vec<serde_json::Value> = vec![];
        assert_eq!(get_values(page), empty);
    }

    {
        // test expected type
        let page = arhiv.list_documents(Filter::default().with_document_type("test_type"))?;

        assert_eq!(get_values(page).len(), 2);
    }

    {
        // test Field
        let page = arhiv.list_documents(Filter::default().where_field("test", "value"))?;

        assert_eq!(get_values(page).len(), 1);
    }

    {
        // test Search
        let page = arhiv.list_documents(Filter::default().search("Val"))?;

        assert_eq!(get_values(page).len(), 2);
    }

    Ok(())
}
