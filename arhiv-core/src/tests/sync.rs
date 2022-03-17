use anyhow::Result;
use serde_json::json;

use rs_utils::workspace_relpath;

use super::utils::*;
use crate::{prime_server::start_prime_server, test_arhiv::TestArhiv};

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = arhiv.get_tx().unwrap();

    let blob_id = tx.add_blob(src, false)?;

    let mut document = empty_document();
    document.data.set("blob", &blob_id);

    tx.stage_document(&mut document)?;
    tx.commit()?;

    assert!(arhiv.get_document(&document.id)?.unwrap().rev.is_staged());

    arhiv.sync().await?;

    assert!(!arhiv.get_document(&document.id)?.unwrap().rev.is_staged());

    // Test blob
    let blob = arhiv.get_blob(&blob_id)?;

    assert!(blob.exists()?);
    assert!(are_equal_files(src, &blob.file_path)?);

    // Test if document is updated correctly
    {
        let tx = arhiv.get_tx().unwrap();

        let mut document = arhiv.get_document(&document.id)?.unwrap();
        document.data = json!({ "test": "other" }).try_into().unwrap();
        tx.stage_document(&mut document)?;

        tx.commit()?;
    }

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().data,
        json!({ "test": "other" }).try_into().unwrap(),
    );

    Ok(())
}

#[tokio::test]
async fn test_replica_sync() -> Result<()> {
    let prime = TestArhiv::new_prime();
    let (join_handle, shutdown_sender, addr) = start_prime_server(prime.0.clone(), 0);
    let replica = TestArhiv::new_replica(addr.port());

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = replica.get_tx().unwrap();

    let blob_id = tx.add_blob(src, false)?;

    let id = {
        let mut document = empty_document();
        document.data.set("blob", &blob_id);
        tx.stage_document(&mut document)?;

        document.id
    };
    tx.commit()?;

    replica.sync().await?;

    assert!(!replica.get_document(&id)?.unwrap().rev.is_staged());

    // Test blob on replica
    {
        let blob = replica.get_blob(&blob_id)?;

        assert!(blob.exists()?);
        assert!(are_equal_files(src, &blob.file_path)?);
    }

    // Test blob on prime
    {
        let blob = prime.get_blob(&blob_id)?;

        assert!(blob.exists()?);
        assert!(are_equal_files(src, &blob.file_path)?);
    }

    // Test if document is updated correctly
    {
        let mut document = replica.get_document(&id)?.unwrap();
        document.data = json!({ "test": "1" }).try_into().unwrap();

        let tx = replica.get_tx().unwrap();
        tx.stage_document(&mut document)?;
        tx.commit()?;

        replica.sync().await?;

        assert_eq!(
            replica.get_document(&id)?.unwrap().data,
            json!({ "test": "1" }).try_into().unwrap(),
        );
    }

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_sync_removes_unused_local_blobs() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let mut tx = arhiv.get_tx().unwrap();

    let blob_id1 = tx.add_blob(&workspace_relpath("resources/k2.jpg"), false)?;

    let mut document = empty_document();
    document.data.set("blob", &blob_id1);

    // stage document with blob1
    tx.stage_document(&mut document)?;

    let blob_id2 = tx.add_blob(&workspace_relpath("resources/text.txt"), false)?;

    document.data.set("blob", &blob_id2);

    // stage document with blob2, blob1 is now unused
    tx.stage_document(&mut document)?;

    tx.commit()?;

    arhiv.sync().await?;

    assert!(!arhiv.get_document(&document.id)?.unwrap().rev.is_staged(),);

    // blob1 should removed
    assert!(!arhiv.get_blob(&blob_id1)?.exists()?);

    // blob2 should be present
    assert!(arhiv.get_blob(&blob_id2)?.exists()?);

    Ok(())
}
