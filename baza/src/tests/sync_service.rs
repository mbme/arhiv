use std::sync::Arc;

use anyhow::Result;
use serde_json::{json, Value};

use rs_utils::workspace_relpath;

use crate::{
    entities::Id,
    sync::{start_rpc_server, BazaClient, SyncAgent, SyncService},
    tests::{are_equal_files, create_changeset, new_document, new_document_snapshot},
    Baza,
};

#[tokio::test]
async fn test_sync_service() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));

    {
        let mut tx = baza0.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 2 })),
        ]))?;
        tx.commit()?;
    }

    let baza1 = Arc::new(Baza::new_test_baza_with_id("1"));
    {
        let mut tx = baza1.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("3", json!({ "1": 2 })),
        ]))?;
        tx.commit()?;
    }

    let mut sync_service = SyncService::new(&baza0);

    let agent1 = SyncAgent::new_in_memory(baza1.clone())?;
    sync_service.add_agent(agent1);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 3);

    assert!(sync_service.sync().await?);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 5);

    Ok(())
}

#[tokio::test]
async fn test_sync_service_fails_on_uncommitted_changes() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));

    {
        let mut tx = baza0.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 2 })),
        ]))?;

        tx.commit()?;
    }

    baza0.add_document(Id::new(), Value::Null)?;

    let sync_service = SyncService::new(&baza0);
    assert!(sync_service.sync().await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_sync_blobs() -> Result<()> {
    let src = &workspace_relpath("resources/k2.jpg");

    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));
    let baza1 = Arc::new(Baza::new_test_baza_with_id("1"));

    let blob_id = {
        let mut tx = baza1.get_tx()?;

        let blob_id = tx.add_blob(src, false)?;

        let mut document = new_document(json!({ "blob": &blob_id }));
        tx.stage_document(&mut document)?;

        tx.commit_staged_documents()?;

        tx.commit()?;

        blob_id
    };

    let mut sync_service = SyncService::new(&baza0);

    let agent1 = SyncAgent::new_in_memory(baza1.clone())?;
    sync_service.add_agent(agent1);

    assert!(sync_service.sync().await?);

    let blob = baza0.get_blob(&blob_id)?;
    let dst = &blob.file_path;
    assert!(are_equal_files(src, dst)?);

    Ok(())
}

#[tokio::test]
async fn test_sync_service_network_agent() -> Result<()> {
    let src = &workspace_relpath("resources/k2.jpg");

    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));

    {
        let mut tx = baza0.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 2 })),
        ]))?;
        tx.commit()?;
    }

    let baza1 = Arc::new(Baza::new_test_baza_with_id("1"));
    {
        let mut tx = baza1.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("3", json!({ "1": 2 })),
        ]))?;
        tx.commit()?;
    }
    let blob_id = {
        let mut tx = baza1.get_tx()?;

        let blob_id = tx.add_blob(src, false)?;

        let mut document = new_document(json!({ "blob": &blob_id }));
        tx.stage_document(&mut document)?;

        tx.commit_staged_documents()?;

        tx.commit()?;

        blob_id
    };

    let mut sync_service = SyncService::new(&baza0);

    let server1 = start_rpc_server(baza1.clone(), 0);

    let client1 = BazaClient::new(server1.get_url()?, &baza1.get_path_manager().downloads_dir);
    let agent1 = SyncAgent::new_in_network(client1);
    sync_service.add_agent(agent1);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 3);

    assert!(sync_service.sync().await?);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 6);

    let blob0 = baza0.get_blob(&blob_id)?;
    assert!(are_equal_files(src, &blob0.file_path)?);

    server1.shutdown().await?;

    Ok(())
}
