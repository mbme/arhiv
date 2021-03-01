use super::utils::*;
use crate::start_prime_server;
use anyhow::*;
use rs_utils::project_relpath;
use serde_json::json;

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let attachment = arhiv.add_attachment(src, true)?;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());

    arhiv.stage_document(document.clone())?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().rev.is_staged(),
        true
    );

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().rev.is_staged(),
        false
    );
    assert_eq!(
        arhiv.get_document(&attachment.id)?.unwrap().rev.is_staged(),
        false
    );

    // Test attachment data
    let attachment_data = arhiv.get_attachment_data_by_id(&attachment.id)?;

    assert_eq!(attachment_data.exists()?, true);
    assert_eq!(are_equal_files(src, &attachment_data.path)?, true);

    // Test if document is updated correctly
    {
        let mut document = arhiv.get_document(&document.id)?.unwrap();
        document.data = json!({ "test": "other" });
        arhiv.stage_document(document)?;
    }

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().data,
        json!({ "test": "other" }),
    );

    Ok(())
}

#[tokio::test]
async fn test_replica_sync() -> Result<()> {
    let prime = new_prime();
    let (join_handle, shutdown_sender, addr) = start_prime_server(prime.clone());
    let replica = new_replica(addr.port());

    let src = &project_relpath("../resources/k2.jpg");

    let attachment = replica.add_attachment(src, true)?;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());
    replica.stage_document(document.clone())?;

    replica.sync().await?;

    assert_eq!(
        replica.get_document(&document.id)?.unwrap().rev.is_staged(),
        false
    );

    // Test attachment data
    {
        let attachment_data = replica.get_attachment_data_by_id(&attachment.id)?;

        assert_eq!(attachment_data.exists()?, true);
        assert_eq!(are_equal_files(src, &attachment_data.path)?, true);
    }

    {
        let attachment_data = prime.get_attachment_data_by_id(&attachment.id)?;

        assert_eq!(attachment_data.exists()?, true);
        assert_eq!(are_equal_files(src, &attachment_data.path)?, true);
    }

    // Test if document is updated correctly
    {
        let mut document = replica.get_document(&document.id)?.unwrap();
        document.data = json!({ "test": "1" });
        replica.stage_document(document)?;
    }

    replica.sync().await?;

    assert_eq!(
        replica.get_document(&document.id)?.unwrap().data,
        json!({ "test": "1" }),
    );

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_sync_removes_unused_local_attachments() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let attachment1 = arhiv.add_attachment(src, true)?;

    let mut document = empty_document();
    document.refs.insert(attachment1.id.clone());

    // stage document with attachment1
    arhiv.stage_document(document.clone())?;

    let attachment2 = arhiv.add_attachment(src, true)?;

    document.refs.clear();
    document.refs.insert(attachment2.id.clone());

    // stage document with attachment2, attachment1 is now unused
    arhiv.stage_document(document.clone())?;

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().rev.is_staged(),
        false
    );

    // attachment1 should removed
    assert_eq!(arhiv.get_document(&attachment1.id)?.is_none(), true);

    // attachment2 should be committed
    assert_eq!(
        arhiv
            .get_document(&attachment2.id)?
            .unwrap()
            .rev
            .is_staged(),
        false
    );

    Ok(())
}
