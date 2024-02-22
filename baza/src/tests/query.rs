use anyhow::Result;
use serde_json::{json, Value};

use rs_utils::workspace_relpath;

use crate::{
    entities::{Document, DocumentClass, Id},
    schema::{DataDescription, DataSchema, Field, FieldType},
    tests::{get_values, new_document},
    BLOBSCount, Baza, DocumentsCount, Filter, OrderBy,
};

#[test]
fn test_pagination() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let tx = baza.get_tx()?;

        baza.add_document(Id::new(), Value::Null)?;
        baza.add_document(Id::new(), Value::Null)?;

        tx.commit()?;
    }

    let page = baza.list_documents(Filter::default().page_size(1))?;

    assert_eq!(page.items.len(), 1);
    assert!(page.has_more);

    Ok(())
}

#[test]
fn test_modes() -> Result<()> {
    let baza = Baza::new_test_baza();

    // committed
    {
        let mut tx = baza.get_tx()?;
        tx.stage_document(&mut new_document(json!({ "test": "1" })), None)?;
        tx.commit_staged_documents()?;
        tx.commit()?;
    }

    // staged
    {
        let mut tx = baza.get_tx()?;
        tx.stage_document(&mut new_document(json!({ "test": "3" })), None)?;
        tx.commit()?;
    }

    {
        // test default
        let page = baza.list_documents(Filter {
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
        let page = baza.list_documents(Filter::default().only_staged())?;

        assert_eq!(get_values(page), vec![json!({ "test": "3" })]);
    }

    Ok(())
}

#[test]
fn test_order_by_enum_field() -> Result<()> {
    let baza = Baza::new_with_schema(DataSchema::new(
        "test",
        vec![DataDescription {
            document_type: "test_type",
            title_format: "title",
            fields: vec![Field {
                name: "enum",
                field_type: FieldType::Enum(&["low", "high", "medium", "other"]),
                mandatory: false,
                readonly: false,
                for_subtypes: None,
            }],
            subtypes: None,
        }],
    ));

    {
        let mut tx = baza.get_tx()?;

        tx.stage_document(&mut new_document(json!({ "enum": "low" })), None)?;
        tx.stage_document(&mut new_document(json!({ "enum": "high" })), None)?;
        tx.stage_document(&mut new_document(json!({ "enum": "other" })), None)?;
        tx.stage_document(&mut new_document(json!({ "enum": "medium" })), None)?;

        tx.commit()?;
    }

    let page = baza.list_documents(Filter {
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
    let baza = Baza::new_with_schema(DataSchema::new(
        "test",
        vec![DataDescription {
            document_type: "test_type",
            title_format: "title",
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
        }],
    ));

    {
        let mut tx = baza.get_tx()?;

        tx.stage_document(
            &mut new_document(json!({ "prop": "b", "other": "2" })),
            None,
        )?;
        tx.stage_document(
            &mut new_document(json!({ "prop": "a", "other": "1" })),
            None,
        )?;
        tx.stage_document(
            &mut new_document(json!({ "prop": "a", "other": "2" })),
            None,
        )?;
        tx.stage_document(
            &mut new_document(json!({ "prop": "b", "other": "1" })),
            None,
        )?;

        tx.commit()?;
    }

    let page = baza.list_documents(Filter {
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

#[test]
fn test_conditions() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let mut tx = baza.get_tx()?;

        tx.stage_document(&mut new_document(json!({ "test": "value" })), None)?;
        tx.stage_document(&mut new_document(json!({ "test": "value1" })), None)?;

        let mut document3 = new_document(json!({ "test": "value2" }));
        tx.stage_document(&mut document3, None)?;
        tx.erase_document(&document3.id)?;

        tx.commit()?;
    }

    {
        // test unexpected type
        let page = baza.list_documents(Filter::default().with_document_type("random"))?;

        let empty: Vec<serde_json::Value> = vec![];
        assert_eq!(get_values(page), empty);
    }

    {
        // test expected type
        let page = baza.list_documents(Filter::default().with_document_type("test_type"))?;

        assert_eq!(get_values(page).len(), 2);
    }

    {
        // test Field
        let page = baza.list_documents(Filter::default().where_field("test", "value"))?;

        assert_eq!(get_values(page).len(), 1);
    }

    {
        // test Search
        let page = baza.list_documents(Filter::default().search("Val"))?;

        assert_eq!(get_values(page).len(), 2);
    }

    {
        // test Search & Document type
        let page = baza.list_documents(
            Filter::default()
                .with_document_type("test_type")
                .search("Val"),
        )?;

        assert_eq!(get_values(page).len(), 2);
    }

    {
        // test Skip erased
        let page = baza.list_documents(Filter::default().skip_erased(true))?;

        assert_eq!(get_values(page).len(), 2);
    }

    {
        // tests only staged
        let page = baza.list_documents(Filter::default().only_staged())?;
        assert_eq!(get_values(page).len(), 3);

        {
            let mut tx = baza.get_tx()?;
            tx.commit_staged_documents()?;
            tx.commit()?;
        }

        let page = baza.list_documents(Filter::default().only_staged())?;
        assert_eq!(get_values(page).len(), 0);
    }

    Ok(())
}

#[test]
fn test_backrefs() -> Result<()> {
    let baza = Baza::new_with_schema(DataSchema::new(
        "test",
        vec![
            //
            DataDescription {
                document_type: "test_type",
                title_format: "title",
                fields: vec![Field {
                    name: "ref",
                    field_type: FieldType::Ref("other_type"),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                }],
                subtypes: None,
            },
            DataDescription {
                document_type: "other_type",
                title_format: "title",
                fields: vec![Field {
                    name: "field",
                    field_type: FieldType::String {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                }],
                subtypes: None,
            },
        ],
    ));

    let mut tx = baza.get_tx()?;

    let mut doc1 = Document::new_with_data(
        DocumentClass::new("other_type", ""),
        json!({ "field": "value" }).try_into().unwrap(),
    );

    tx.stage_document(&mut doc1, None)?;

    tx.stage_document(
        &mut Document::new_with_data(
            DocumentClass::new("test_type", ""),
            json!({
                "ref": &doc1.id,
            })
            .try_into()
            .unwrap(),
        ),
        None,
    )?;
    tx.stage_document(
        &mut Document::new_with_data(
            DocumentClass::new("test_type", ""),
            json!({
                "ref": &doc1.id,
            })
            .try_into()
            .unwrap(),
        ),
        None,
    )?;

    tx.commit()?;

    let page = baza.list_documents(Filter::all_backrefs(doc1.id))?;

    assert_eq!(page.items.len(), 2);

    Ok(())
}

#[test]
fn test_collections() -> Result<()> {
    let baza = Baza::new_with_schema(DataSchema::new(
        "test",
        vec![
            //
            DataDescription {
                document_type: "collection_type",
                title_format: "title",
                fields: vec![Field {
                    name: "items",
                    field_type: FieldType::RefList("other_type"),
                    mandatory: true,
                    readonly: false,
                    for_subtypes: None,
                }],
                subtypes: None,
            },
            DataDescription {
                document_type: "other_type",
                title_format: "title",
                fields: vec![Field {
                    name: "field",
                    field_type: FieldType::String {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: None,
                }],
                subtypes: None,
            },
        ],
    ));

    let mut tx = baza.get_tx()?;

    let mut doc1 = Document::new_with_data(
        DocumentClass::new("other_type", ""),
        json!({ "field": "value" }).try_into().unwrap(),
    );

    tx.stage_document(&mut doc1, None)?;

    tx.stage_document(
        &mut Document::new_with_data(
            DocumentClass::new("collection_type", ""),
            json!({
                "items": vec![&doc1.id],
            })
            .try_into()
            .unwrap(),
        ),
        None,
    )?;
    tx.stage_document(
        &mut Document::new_with_data(
            DocumentClass::new("collection_type", ""),
            json!({
                "items": vec![&doc1.id],
            })
            .try_into()
            .unwrap(),
        ),
        None,
    )?;

    tx.commit()?;

    let page = baza.list_documents(Filter::all_collections(doc1.id))?;

    assert_eq!(page.items.len(), 2);

    Ok(())
}

#[allow(clippy::too_many_lines)]
#[test]
fn test_count_documents_and_blobs() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let documents_count = baza.get_connection()?.count_documents()?;
        assert_eq!(
            documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 0,
                documents_new: 0,
                erased_documents_committed: 0,
                erased_documents_staged: 0,
                snapshots: 0,
            }
        );

        let blobs_count = baza.get_connection()?.count_blobs()?;
        assert_eq!(
            blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 0,
                local_blobs_count: 0,
                local_used_blobs_count: 0,
            }
        );
    }

    // create document with blob
    let mut document = {
        let mut tx = baza.get_tx()?;

        let blob_id = tx.add_blob(&workspace_relpath("resources/k2.jpg"), false)?;
        let mut document = new_document(json!({
            "test": "test",
            "blob": blob_id,
        }));
        tx.stage_document(&mut document, None)?;

        tx.commit()?;

        document
    };

    {
        let blobs_count = baza.get_connection()?.count_blobs()?;
        assert_eq!(
            blobs_count,
            BLOBSCount {
                blobs_staged: 1,
                total_blobs_count: 1,
                local_blobs_count: 1,
                local_used_blobs_count: 1,
            }
        );
    }

    // commit document
    {
        let mut tx = baza.get_tx()?;
        tx.commit_staged_documents()?;
        tx.commit()?;
    }

    {
        let documents_count = baza.get_connection()?.count_documents()?;
        assert_eq!(
            documents_count,
            DocumentsCount {
                documents_committed: 1,
                documents_updated: 0,
                documents_new: 0,
                erased_documents_committed: 0,
                erased_documents_staged: 0,
                snapshots: 1,
            }
        );

        let blobs_count = baza.get_connection()?.count_blobs()?;
        assert_eq!(
            blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 1,
                local_blobs_count: 1,
                local_used_blobs_count: 1,
            }
        );
    }

    {
        let mut tx = baza.get_tx()?;

        // update document
        tx.stage_document(&mut document, None)?;

        // create another document
        tx.stage_document(&mut new_document(json!({ "test": "test" })), None)?;

        tx.commit()?;
    }

    {
        let documents_count = baza.get_connection()?.count_documents()?;

        assert_eq!(
            documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 1,
                documents_new: 1,
                erased_documents_committed: 0,
                erased_documents_staged: 0,
                snapshots: 3,
            }
        );

        assert_eq!(documents_count.count_staged_documents(), 2);
    }

    // delete document
    {
        let tx = baza.get_tx()?;

        tx.erase_document(&document.id)?;

        tx.commit()?;
    }

    {
        let documents_count = baza.get_connection()?.count_documents()?;
        assert_eq!(
            documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 0,
                documents_new: 1,
                erased_documents_committed: 0,
                erased_documents_staged: 1,
                snapshots: 3,
            }
        );

        let blobs_count = baza.get_connection()?.count_blobs()?;
        assert_eq!(
            blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 0,
                local_blobs_count: 1,
                local_used_blobs_count: 0,
            }
        );
    }

    {
        let mut tx = baza.get_tx()?;
        tx.commit_staged_documents()?;
        tx.commit()?;
    }

    {
        let documents_count = baza.get_connection()?.count_documents()?;
        assert_eq!(
            documents_count,
            DocumentsCount {
                documents_committed: 1,
                documents_updated: 0,
                documents_new: 0,
                erased_documents_committed: 1,
                erased_documents_staged: 0,
                snapshots: 2,
            }
        );

        let blobs_count = baza.get_connection()?.count_blobs()?;
        assert_eq!(
            blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 0,
                local_blobs_count: 0,
                local_used_blobs_count: 0,
            }
        );
    }

    Ok(())
}
