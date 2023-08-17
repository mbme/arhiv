use anyhow::Result;
use serde_json::json;

use crate::{
    entities::Id,
    sync::Revision,
    tests::{create_changeset, new_document_snapshot},
    Baza,
};

#[test]
fn test_sync_get_db_rev() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::new(), json!({}))?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 2 }))?;
    baza.add_document(Id::new(), json!({ "0": 2, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 3, "1": 2 }))?;

    {
        let tx = baza.get_tx()?;

        assert_eq!(
            tx.get_db_rev()?,
            Revision::from_value(json!({ "0": 3, "1": 2 }))?
        );
    }

    Ok(())
}

#[test]
fn test_sync_get_changeset() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::new(), json!({ "0": 1, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 2 }))?;
    baza.add_document(Id::new(), json!({ "0": 2, "1": 1 }))?;

    {
        let mut tx = baza.get_tx()?;
        let changeset = tx.get_changeset(&Revision::from_value(json!({ "0": 1 }))?)?;

        assert_eq!(changeset.documents.len(), 3);
    }

    {
        let mut tx = baza.get_tx()?;
        let changeset = tx.get_changeset(&Revision::from_value(json!({ "0": 1, "1": 1 }))?)?;

        assert_eq!(changeset.documents.len(), 2);
    }

    {
        baza.add_document(Id::new(), json!({}))?;

        let mut tx = baza.get_tx()?;
        let changeset = tx.get_changeset(&Revision::from_value(json!({ "0": 1, "1": 1 }))?);

        assert!(changeset.is_err());
    }

    Ok(())
}

#[test]
fn test_sync_apply_changeset() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 3 })),
        ]))?;

        let revisions = tx.list_all_document_snapshots()?;
        assert_eq!(revisions.len(), 3);
    }

    {
        let mut tx = baza.get_tx()?;
        let mut changeset = create_changeset(vec![
            new_document_snapshot("2", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("2", json!({ "0": 2, "1": 1 })),
        ]);
        changeset.documents[2].erase();

        tx.apply_changeset(changeset)?;

        let revisions = tx.list_all_document_snapshots()?;
        assert_eq!(revisions.len(), 2);
    }

    Ok(())
}

#[test]
fn test_sync_get_conflicting_documents() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 2 })),
        ]))?;

        let ids = tx.get_coflicting_documents()?;
        assert_eq!(ids.len(), 0);

        tx.commit()?;
    }

    {
        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("2", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("2", json!({ "0": 2, "1": 1 })),
        ]))?;

        let ids = tx.get_coflicting_documents()?;
        assert_eq!(ids.len(), 1);

        tx.commit()?;
    }

    {
        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("3", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("3", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("3", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("3", json!({})),
        ]))?;

        let ids = tx.get_coflicting_documents()?;
        assert_eq!(ids.len(), 2);

        tx.commit()?;
    }

    Ok(())
}
