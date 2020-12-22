use anyhow::*;
use arhiv::*;
use serde_json::{json, Value};
pub use utils::*;

mod utils;

#[test]
fn test_pagination() -> Result<()> {
    let arhiv = new_prime();

    arhiv.stage_document(empty_document(), vec![])?;
    arhiv.stage_document(empty_document(), vec![])?;

    let page = arhiv.list_documents(DocumentFilter {
        page_size: Some(1),
        ..DocumentFilter::default()
    })?;

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.has_more, true);

    Ok(())
}

#[test]
fn test_order_by_enum_field() -> Result<()> {
    let arhiv = new_prime();

    arhiv.stage_document(new_document(json!({ "enum": "low" })), vec![])?;
    arhiv.stage_document(new_document(json!({ "enum": "high" })), vec![])?;
    arhiv.stage_document(new_document(json!({ "enum": "other" })), vec![])?;
    arhiv.stage_document(new_document(json!({ "enum": "medium" })), vec![])?;

    let page = arhiv.list_documents(DocumentFilter {
        order: vec![OrderBy::EnumField {
            selector: "$.enum".to_string(),
            asc: true,
            enum_order: vec!["high".to_string(), "medium".to_string(), "low".to_string()],
        }],
        ..DocumentFilter::default()
    })?;

    assert_eq!(
        page.items
            .into_iter()
            .map(|item| item.data)
            .collect::<Vec<Value>>(),
        vec![
            json!({ "enum": "high" }),
            json!({ "enum": "medium" }),
            json!({ "enum": "low" }),
            json!({ "enum": "other" }),
        ]
    );

    Ok(())
}
