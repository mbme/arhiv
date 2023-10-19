use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;

use crate::{entities::Id, sync::SyncManager, Baza};

#[test]
fn test_locks() -> Result<()> {
    let baza = Baza::new_test_baza();

    let id = Id::from("1");
    baza.add_document(id.clone(), Value::Null)?;

    let mut tx = baza.get_tx()?;

    assert!(tx.list_locks()?.is_empty());

    tx.lock_document(&id, "test".to_string())?;

    assert!(tx.list_locks()?.contains_key(&id));

    // if can modify document when locked
    let mut document = tx.get_document(&id)?.unwrap();
    tx.stage_document(&mut document)?;

    assert!(tx.unlock_document(&id)?);
    assert!(tx.list_locks()?.is_empty());

    tx.commit()?;

    Ok(())
}

#[test]
fn test_lock_blocks_commit() -> Result<()> {
    let baza = Baza::new_test_baza();

    let id = Id::from("1");
    baza.add_document(id.clone(), Value::Null)?;

    let mut tx = baza.get_tx()?;

    tx.lock_document(&id, "test".to_string())?;

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
        tx.lock_document(&id, "test".to_string())?;
        tx.commit()?;
    }

    let mut sync_manager0 = SyncManager::new(baza0.clone())?;

    let baza1 = Arc::new(Baza::new_test_baza());
    sync_manager0.add_in_mem_agent(baza1)?;

    assert!(!sync_manager0.sync().await?);

    Ok(())
}
