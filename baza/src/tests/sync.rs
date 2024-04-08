use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::Extension;
use serde_json::{json, Value};
use tokio::time::{advance, sleep};

use rs_utils::{http_server::HttpServer, workspace_relpath};

use crate::{
    entities::{Id, Revision},
    sync::{build_rpc_router, SyncManager},
    tests::{
        are_equal_files, create_changeset, new_certificate, new_document, new_document_snapshot,
    },
    Baza, Credentials,
};

async fn start_rpc_server(baza: Arc<Baza>) -> HttpServer {
    let certificate = new_certificate();
    let router = build_rpc_router(certificate.certificate_der.clone())
        .expect("must create RPC router")
        .layer(Extension(baza));

    HttpServer::new_https(0, router, certificate)
        .await
        .expect("must start rpc server")
}

#[test]
fn test_sync_get_db_rev() -> Result<()> {
    let baza = Baza::new_test_baza();

    baza.add_document(Id::new(), Value::Null)?;
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
        let tx = baza.get_tx()?;
        let changeset = tx.get_changeset(&Revision::from_value(json!({ "0": 1 }))?)?;

        assert_eq!(changeset.documents.len(), 3);
    }

    {
        let tx = baza.get_tx()?;
        let changeset = tx.get_changeset(&Revision::from_value(json!({ "0": 1, "1": 1 }))?)?;

        assert_eq!(changeset.documents.len(), 2);
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

    // There should be no conflict if document revisions aren't concurrent
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

    // There should be conflict if document revisions are concurrent
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

    // Conflict should exist even if there is a staged document
    {
        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("3", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("3", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("3", json!({ "0": 2, "1": 1 })),
        ]))?;

        let mut document = tx.must_get_document(&Id::from("3"))?;
        tx.stage_document(&mut document, None)?;

        let ids = tx.get_coflicting_documents()?;
        assert_eq!(ids.len(), 2);

        tx.commit()?;
    }

    // Conflict should get resolved after new document snapshot gets committed
    {
        let mut tx = baza.get_tx()?;

        tx.commit_staged_documents()?;

        let ids = tx.get_coflicting_documents()?;
        assert_eq!(ids.len(), 1);

        tx.commit()?;
    }

    Ok(())
}

#[tokio::test]
async fn test_sync() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));
    let mut sync_manager0 = SyncManager::new(baza0.clone());

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

    sync_manager0.add_in_mem_agent(baza1.clone())?;
    sync_manager0.add_in_mem_agent(baza1)?;
    assert_eq!(
        sync_manager0.count_agents(),
        1,
        "sync manager must keep only one agent per instance_id"
    );

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 3);

    assert!(sync_manager0.sync().await?);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 5);

    Ok(())
}

#[tokio::test]
async fn test_sync_fails_on_uncommitted_changes() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));
    let sync_manager0 = SyncManager::new(baza0.clone());

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

    assert!(sync_manager0.sync().await.is_ok_and(|synced| !synced));

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
        tx.stage_document(&mut document, None)?;

        tx.commit_staged_documents()?;

        tx.commit()?;

        blob_id
    };

    let mut sync_manager0 = SyncManager::new(baza0.clone());
    sync_manager0.add_in_mem_agent(baza1)?;

    assert!(sync_manager0.sync().await?);

    let blob = baza0.get_blob(&blob_id)?;
    let dst = &blob.file_path;
    assert!(are_equal_files(src, dst)?);

    Ok(())
}

#[tokio::test]
async fn test_sync_network_agent_success() -> Result<()> {
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
        tx.stage_document(&mut document, None)?;

        tx.commit_staged_documents()?;

        tx.commit()?;

        blob_id
    };

    let sync_manager0 = SyncManager::new(baza0.clone());

    let server1 = start_rpc_server(baza1.clone()).await;
    sync_manager0.add_network_agent(
        baza1.get_connection()?.get_instance_id()?,
        server1.get_url()?.as_str(),
        &new_certificate(),
    )?;

    let snapshots_count = baza0.get_connection()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 3);

    assert!(sync_manager0.sync().await?);

    let snapshots_count = baza0.get_connection()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 6);

    let blob0 = baza0.get_blob(&blob_id)?;
    assert!(are_equal_files(src, &blob0.file_path)?);

    sync_manager0.remove_all_agents(); // clears network connections pool
    server1.shutdown().await?;

    Ok(())
}

#[tokio::test]
async fn test_sync_network_agent_fails_with_wrong_auth() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));
    baza0.update_credentials(Credentials::new(
        baza0.get_connection()?.get_login()?,
        "other password".to_string(),
    )?)?;

    let baza1 = Arc::new(Baza::new_test_baza_with_id("1"));
    {
        let mut tx = baza1.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 2 })),
        ]))?;
        tx.commit()?;
    }

    let sync_manager0 = SyncManager::new(baza0.clone());

    let server1 = start_rpc_server(baza1.clone()).await;
    sync_manager0.add_network_agent(
        baza1.get_connection()?.get_instance_id()?,
        server1.get_url()?.as_str(),
        &new_certificate(),
    )?;

    assert!(!sync_manager0.sync().await?);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 0);

    sync_manager0.remove_all_agents(); // clears network connections pool
    server1.shutdown().await?;

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_auto_sync_on_commit() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza());
    let baza1 = Arc::new(Baza::new_test_baza());
    baza1.add_document(Id::new(), json!({ "0": 3, "1": 2 }))?;

    let mut sync_manager0 = SyncManager::new(baza0.clone());
    sync_manager0.add_in_mem_agent(baza1.clone())?;
    let sync_manager0 = Arc::new(sync_manager0);

    let auto_sync_delay = Duration::from_secs(10);
    sync_manager0.clone().start_auto_sync(auto_sync_delay)?;

    sleep(Duration::from_secs(1)).await;

    {
        let mut tx = baza0.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document, None)?;
        tx.commit_staged_documents()?;
        tx.commit()?;
    }

    let snapshots_count = baza0.get_connection()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 1);

    sleep(Duration::from_secs(1)).await;
    advance(auto_sync_delay).await;
    sleep(Duration::from_secs(1)).await;

    let snapshots_count = baza0.get_connection()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 2);

    Ok(())
}
