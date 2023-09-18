use anyhow::Result;
use serde_json::{json, Value};

use rs_utils::{workspace_relpath, TempFile};

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
        let tx = baza.get_connection()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();
        assert_eq!(document.rev, None,);
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

        let mut document = tx.get_document(&Id::from("1"))?.unwrap();
        assert_eq!(
            document.get_rev()?,
            &Revision::from_value(json!({ "0": 2, "2": 1 }))?
        );

        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    {
        let tx = baza.get_connection()?;

        let document = tx.get_document(&Id::from("1"))?.unwrap();
        assert!(document.is_staged());
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

    let mut tx = baza.get_tx()?;

    let mut document = new_document(json!({}));
    tx.stage_document(&mut document)?;

    assert_eq!(tx.get_db_rev()?, Revision::initial());

    let committed_count = tx.commit_staged_documents()?;

    assert_eq!(committed_count, 1);

    assert_eq!(tx.get_db_rev()?, Revision::from_value(json!({ "0": 1 }))?);

    Ok(())
}

#[test]
fn test_crud_commit_deduce_version() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::new(), Value::Null)?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 1, "1": 2 }))?;
    baza.add_document(Id::new(), json!({ "0": 2, "1": 1 }))?;
    baza.add_document(Id::new(), json!({ "0": 2, "1": 3 }))?;

    {
        let mut tx = baza.get_tx()?;

        let mut document = new_document(json!({}));
        tx.stage_document(&mut document)?;
        tx.commit_staged_documents()?;

        let document = tx.must_get_document(&document.id)?;

        assert_eq!(
            document.get_rev()?,
            &Revision::from_value(json!({ "0": 3, "1": 3 }))?
        );
    }

    Ok(())
}

#[test]
fn test_crud_add_blob() -> Result<()> {
    let baza = Baza::new_test_baza();

    {
        let tx = baza.get_tx()?;

        let mut document = new_document(json!({}));
        document.id = Id::from("1");
        tx.stage_document(&mut document)?;

        tx.commit()?;
    }

    let temp_dir = TempFile::new_with_details("test_add_blob_soft_links_and_dirs", "");
    temp_dir.mkdir()?;

    let resource_file = workspace_relpath("resources/k2.jpg");
    let resource_file_link = format!("{temp_dir}/resource_file_link");
    std::os::unix::fs::symlink(resource_file, &resource_file_link)?;

    {
        let mut tx = baza.get_tx()?;
        let result = tx.add_blob(&resource_file_link, false);
        assert!(result.is_err());
    }

    let resource_dir = workspace_relpath("resources");
    let resource_dir_link = format!("{}/resource_dir_link", &temp_dir);
    std::os::unix::fs::symlink(&resource_dir, &resource_dir_link)?;

    {
        let mut tx = baza.get_tx()?;
        let result = tx.add_blob(&resource_dir, false);
        assert!(result.is_err());
    }

    {
        let mut tx = baza.get_tx()?;
        let result = tx.add_blob(&resource_dir_link, false);
        assert!(result.is_err());
    }

    Ok(())
}

#[test]
fn test_crud_remove_orphaned_blob() -> Result<()> {
    let baza = Baza::new_test_baza();

    let document_id = {
        let mut tx = baza.get_tx()?;

        tx.add_blob(&workspace_relpath("resources/karpaty.jpeg"), false)?;

        let blob_id = tx.add_blob(&workspace_relpath("resources/k2.jpg"), false)?;

        let mut document = new_document(json!({ "blob": blob_id }));
        tx.stage_document(&mut document)?;

        tx.commit()?;

        document.id
    };

    assert_eq!(baza.get_connection()?.get_local_blob_ids()?.len(), 2);

    {
        let mut tx = baza.get_tx()?;
        tx.commit_staged_documents()?;
        tx.commit()?;
    }

    assert_eq!(baza.get_connection()?.get_local_blob_ids()?.len(), 1);

    {
        let mut tx = baza.get_tx()?;
        tx.erase_document(&document_id)?;
        tx.commit_staged_documents()?;
        tx.commit()?;
    }

    assert_eq!(baza.get_connection()?.get_local_blob_ids()?.len(), 0);

    Ok(())
}
