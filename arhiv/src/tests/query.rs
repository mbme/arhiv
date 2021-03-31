use super::utils::*;
use crate::*;
use anyhow::*;
use serde_json::json;

#[test]
fn test_pagination() -> Result<()> {
    let arhiv = new_prime();

    arhiv.stage_document(empty_document())?;
    arhiv.stage_document(empty_document())?;

    let page = arhiv.list_documents(Filter {
        page_size: Some(1),
        ..Filter::default()
    })?;

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.has_more, true);

    Ok(())
}

#[tokio::test]
async fn test_modes() -> Result<()> {
    let arhiv = new_prime();

    // committed
    arhiv.stage_document(new_document(json!("1")))?;

    {
        // archived
        let mut doc = new_document(json!("2"));
        doc.archived = true;
        arhiv.stage_document(doc)?;
    }

    arhiv.sync().await?;

    // staged
    arhiv.stage_document(new_document(json!("3")))?;

    {
        // test default
        let page = arhiv.list_documents(Filter {
            order: vec![OrderBy::UpdatedAt { asc: false }],
            ..Filter::default()
        })?;

        assert_eq!(get_values(page), vec![json!("3"), json!("1"),]);
    }

    {
        // test archived
        let page = arhiv.list_documents(Filter {
            mode: Some(FilterMode::Archived),
            ..Filter::default()
        })?;

        assert_eq!(get_values(page), vec![json!("2")]);
    }

    {
        // test staged
        let page = arhiv.list_documents(Filter {
            mode: Some(FilterMode::Staged),
            ..Filter::default()
        })?;

        assert_eq!(get_values(page), vec![json!("3")]);
    }

    Ok(())
}

#[test]
fn test_order_by_enum_field() -> Result<()> {
    let arhiv = new_prime();

    arhiv.stage_document(new_document(json!({ "enum": "low" })))?;
    arhiv.stage_document(new_document(json!({ "enum": "high" })))?;
    arhiv.stage_document(new_document(json!({ "enum": "other" })))?;
    arhiv.stage_document(new_document(json!({ "enum": "medium" })))?;

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
    let arhiv = new_prime();

    arhiv.stage_document(new_document(json!({ "prop": "b", "other": "2" })))?;
    arhiv.stage_document(new_document(json!({ "prop": "a", "other": "1" })))?;
    arhiv.stage_document(new_document(json!({ "prop": "a", "other": "2" })))?;
    arhiv.stage_document(new_document(json!({ "prop": "b", "other": "1" })))?;

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
    let arhiv = new_prime();
    arhiv.stage_document(new_document(json!({ "test": "value" })))?;
    arhiv.stage_document(new_document(json!({ "test": "value1" })))?;

    {
        // test unexpected type
        let page = arhiv.list_documents(Filter {
            matchers: vec![Matcher::Type {
                document_type: "random".to_string(),
            }],
            ..Filter::default()
        })?;

        let empty: Vec<serde_json::Value> = vec![];
        assert_eq!(get_values(page), empty);
    }

    {
        // test expected type
        let page = arhiv.list_documents(Filter {
            matchers: vec![Matcher::Type {
                document_type: "test_type".to_string(),
            }],
            ..Filter::default()
        })?;

        assert_eq!(get_values(page).len(), 2);
    }

    {
        // test Field

        let page = arhiv.list_documents(Filter {
            matchers: vec![Matcher::Field {
                selector: "$.test".to_string(),
                pattern: "value".to_string(),
            }],
            ..Filter::default()
        })?;

        assert_eq!(get_values(page).len(), 1);
    }

    {
        // test Search

        let page = arhiv.list_documents(Filter {
            matchers: vec![Matcher::Search {
                pattern: "Val*".to_string(),
            }],
            ..Filter::default()
        })?;

        assert_eq!(get_values(page).len(), 2);
    }

    Ok(())
}
