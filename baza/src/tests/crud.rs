use anyhow::Result;
use serde_json::{json, Value};

use crate::{entities::Id, sync::Revision, Baza};

use super::new_document;

#[test]
fn test_crud_create() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let tx = baza.get_tx()?;

        let mut document = new_document(json!({}));
        document.id = Id::from("1");
        tx.stage_document(&mut document)?;

        tx.commit()?;
    }

    {
        let tx = baza.get_tx()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();
        assert_eq!(document.rev, Revision::from_value(json!({ "0": 0 }))?);
    }

    Ok(())
}

#[test]
fn test_crud_read() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::from("1"), json!({ "0": 1, "2": 1 }))?;
    baza.add_document(Id::from("1"), json!({ "0": 2, "2": 1 }))?;

    {
        let tx = baza.get_tx()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();
        assert_eq!(
            document.rev,
            Revision::from_value(json!({ "0": 2, "2": 1 }))?
        );
    }

    baza.add_document(Id::from("1"), json!({}))?;

    {
        let tx = baza.get_tx()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();
        assert_eq!(document.rev, Revision::from_value(json!({}))?);
    }

    Ok(())
}

#[test]
fn test_crud_update() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::from("1"), json!({ "0": 1 }))?;

    {
        let tx = baza.get_tx()?;

        let mut document = tx.get_document(&Id::from("1"))?.unwrap();
        document.data.set("test", "value");
        tx.stage_document(&mut document)?;

        tx.commit()?;
    }

    {
        let tx = baza.get_tx()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();
        assert_eq!(
            Into::<Value>::into(document.data),
            json!({ "test": "value" })
        );
    }

    Ok(())
}

#[test]
fn test_crud_delete() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::from("1"), json!({ "0": 1 }))?;
    baza.add_document(Id::from("1"), json!({ "0": 2 }))?;

    {
        let tx = baza.get_tx()?;

        tx.erase_document(&Id::from("1"))?;
        tx.commit()?;
    }

    {
        let tx = baza.get_tx()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();

        assert!(document.is_erased());

        let has_snapshot =
            tx.has_snapshot(&Id::from("1"), &Revision::from_value(json!({ "0": 1 }))?)?;
        assert!(has_snapshot);
    }

    Ok(())
}

#[test]
fn test_crud_commit() -> Result<()> {
    let baza = Baza::new_test_baza();

    let tx = baza.get_tx()?;

    let mut document = new_document(json!({}));
    tx.stage_document(&mut document)?;

    assert_eq!(tx.get_db_rev()?, Revision::from_value(json!({ "0": 0 }))?);

    let committed_count = tx.commit_staged_documents()?;

    assert_eq!(committed_count, 1);

    assert_eq!(tx.get_db_rev()?, Revision::from_value(json!({ "0": 1 }))?);

    Ok(())
}

#[test]
fn test_crud_commit_deduce_version() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::new(), json!({}))?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 2 }))?;
    baza.add_document(Id::new(), json!({ "0": 2, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 2, "1": 3 }))?;

    {
        let tx = baza.get_tx()?;

        let mut document = new_document(json!({}));
        tx.stage_document(&mut document)?;
        tx.commit_staged_documents()?;

        let document = tx.must_get_document(&document.id)?;

        assert_eq!(
            document.rev,
            Revision::from_value(json!({ "0": 3, "1": 3 }))?
        );
    }

    Ok(())
}
