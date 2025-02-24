use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;

use crate::{
    entities::{DocumentLockKey, Id},
    sync::SyncManager,
    Baza,
};

#[test]
fn test_locks() -> Result<()> {
    let baza = Baza::new_test_baza();

    let id = Id::from("1");
    baza.add_document(id.clone(), Value::Null)?;

    let mut tx = baza.get_tx()?;

    assert!(tx.list_document_locks()?.is_empty());

    let lock = tx.lock_document(&id, "test")?;

    assert!(tx.list_document_locks()?.contains_key(&id));

    // double lock returns an error
    assert!(tx.lock_document(&id, "test").is_err());

    assert!(tx.unlock_document(&id, lock.get_key()).is_ok());
    assert!(tx.list_document_locks()?.is_empty());

    tx.commit()?;

    Ok(())
}

#[test]
fn test_lock_stage_document() -> Result<()> {
    let baza = Baza::new_test_baza();

    let id = Id::from("1");
    baza.add_document(id.clone(), Value::Null)?;

    let mut tx = baza.get_tx()?;

    let lock = tx.lock_document(&id, "test")?;

    // if can modify locked document without a key
    {
        let mut document = tx.get_document(&id)?.unwrap();
        assert!(tx.stage_document(&mut document, None).is_err());
    }

    // if can modify locked document with an invalid key
    {
        let mut document = tx.get_document(&id)?.unwrap();
        assert!(tx
            .stage_document(
                &mut document,
                Some(DocumentLockKey::from_string("invalid key"))
            )
            .is_err());
    }

    // if can erase locked document
    assert!(tx.erase_document(&id).is_err());

    // if can modify locked document with a valid key
    {
        let mut document = tx.get_document(&id)?.unwrap();
        assert!(tx
            .stage_document(&mut document, Some(lock.get_key().clone()))
            .is_ok());
    }

    tx.unlock_document(&id, lock.get_key())?;

    // if can modify unlocked document without a key
    {
        let mut document = tx.get_document(&id)?.unwrap();
        assert!(tx.stage_document(&mut document, None).is_ok());
    }

    // if can modify unlocked document with an invalid key
    {
        let mut document = tx.get_document(&id)?.unwrap();
        assert!(tx
            .stage_document(
                &mut document,
                Some(DocumentLockKey::from_string("invalid key"))
            )
            .is_err());
    }

    tx.commit()?;

    Ok(())
}

#[test]
fn test_lock_blocks_commit() -> Result<()> {
    let baza = Baza::new_test_baza();

    let id = Id::from("1");
    baza.add_document(id.clone(), Value::Null)?;

    let mut tx = baza.get_tx()?;

    tx.lock_document(&id, "test")?;

    assert!(tx.commit_staged_documents().is_err());

    tx.commit()?;

    Ok(())
}

#[tokio::test]
async fn test_lock_blocks_sync() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza());

    let id = Id::from("1");
    baza0.add_document(id.clone(), Value::Null)?;

    {
        let mut tx = baza0.get_tx()?;
        tx.commit_staged_documents()?;
        tx.lock_document(&id, "test")?;
        tx.commit()?;
    }

    let mut sync_manager0 = SyncManager::new(baza0.clone())?;

    let baza1 = Arc::new(Baza::new_test_baza());
    sync_manager0.add_in_mem_agent(baza1)?;

    assert!(!sync_manager0.sync().await?);

    Ok(())
}
