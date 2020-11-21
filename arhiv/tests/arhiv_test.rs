use anyhow::*;
use arhiv::entities::*;
use arhiv::{start_server, Arhiv, DocumentFilter};
use rs_utils::project_relpath;
use serde_json::json;
use std::sync::Arc;
use utils::*;

mod utils;

fn test_crud(arhiv: &Arhiv) -> Result<()> {
    // CREATE
    let mut document = new_document();
    document.data = json!({ "test": "test" });
    arhiv.stage_document(document.clone(), vec![])?;
    assert_eq!(arhiv.list_documents(None)?.items.len(), 1);

    // READ
    {
        let other_document = arhiv.get_document(&document.id)?.unwrap();

        assert_eq!(other_document.data, document.data);
        assert_eq!(other_document.rev.is_staged(), true);
    }

    // UPDATE
    {
        let mut other_document = arhiv.get_document(&document.id)?.unwrap();
        other_document.data = json!({ "test": "1" });
        arhiv.stage_document(other_document.clone(), vec![])?;

        assert_eq!(
            arhiv.get_document(&document.id)?.unwrap().data,
            other_document.data
        );
    }

    // DELETE
    {
        assert_eq!(arhiv.list_documents(None)?.items.len(), 1);
        let mut other_document = arhiv.get_document(&document.id)?.unwrap();
        other_document.archived = true;
        arhiv.stage_document(other_document, vec![])?;

        assert_eq!(arhiv.get_document(&document.id)?.unwrap().archived, true);
        assert_eq!(arhiv.list_documents(None)?.items.len(), 0);
    }

    Ok(())
}

#[test]
fn test_prime_crud() -> Result<()> {
    test_crud(&new_prime())
}

#[test]
fn test_replica_crud() -> Result<()> {
    test_crud(&new_replica())
}

#[test]
fn test_pagination() -> Result<()> {
    let arhiv = new_prime();

    arhiv.stage_document(new_document(), vec![])?;
    arhiv.stage_document(new_document(), vec![])?;

    let page = arhiv.list_documents(Some(DocumentFilter {
        page_size: Some(1),
        ..DocumentFilter::default()
    }))?;

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.has_more, true);

    Ok(())
}

fn test_attachments(arhiv: &Arhiv) -> Result<()> {
    assert_eq!(arhiv.list_attachments(None)?.items.len(), 0);

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = new_document();
    document.refs.insert(attachment.id.clone());

    arhiv.stage_document(document, vec![attachment.clone()])?;
    assert_eq!(
        arhiv
            .get_attachment_data(&attachment.id)
            .staged_file_exists()?,
        true
    );

    let dst = &arhiv
        .get_attachment_data(&attachment.id)
        .get_staged_file_path();

    assert_eq!(arhiv.list_attachments(None)?.items.len(), 1);
    assert_eq!(are_equal_files(src, dst)?, true);

    Ok(())
}

#[test]
fn test_prime_attachments() -> Result<()> {
    test_attachments(&new_prime())
}

#[test]
fn test_replica_attachments() -> Result<()> {
    test_attachments(&new_replica())
}

#[tokio::test]
async fn test_update_attachment_filename() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = new_document();
    document.refs.insert(attachment.id.clone());

    arhiv.stage_document(document, vec![attachment.clone()])?;

    let attachment = arhiv.get_attachment(&attachment.id)?.unwrap();
    assert_eq!(attachment.filename, "k2.jpg");

    arhiv.update_attachment_filename(&attachment.id, "k1.jpg")?;
    assert_eq!(
        arhiv.get_attachment(&attachment.id)?.unwrap().filename,
        "k1.jpg"
    );

    assert_eq!(arhiv.get_status()?.rev.0, 0);

    arhiv.sync().await?;

    assert_eq!(arhiv.get_status()?.rev.0, 1);

    // make sure we increase rev after updating committed filename
    arhiv.update_attachment_filename(&attachment.id, "k1.jpg")?;
    assert_eq!(
        arhiv.get_attachment(&attachment.id)?.unwrap().filename,
        "k1.jpg"
    );
    assert_eq!(arhiv.get_status()?.rev.0, 2);

    Ok(())
}

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut other_attachment = AttachmentSource::new(src);
    other_attachment.copy = true;

    let mut document = new_document();
    document.refs.insert(other_attachment.id.clone());

    arhiv.stage_document(document.clone(), vec![other_attachment.clone()])?;

    // now replace attachment ref with other_attachment ref
    document.refs.clear();
    document.refs.insert(attachment.id.clone());
    arhiv.stage_document(document.clone(), vec![attachment.clone()])?;
    // so that attachment is unused now

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().rev.is_staged(),
        true
    );

    assert_eq!(
        arhiv
            .get_attachment(&other_attachment.id)?
            .unwrap()
            .rev
            .is_staged(),
        true
    );

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().rev.is_staged(),
        false
    );
    assert_eq!(
        arhiv
            .get_attachment(&attachment.id)?
            .unwrap()
            .rev
            .is_staged(),
        false
    );

    // make sure unused attachment wasn't committed
    assert_eq!(
        arhiv
            .get_attachment(&other_attachment.id)?
            .unwrap()
            .rev
            .is_staged(),
        true
    );

    // Test attachment data
    let data = arhiv.get_attachment_data(&attachment.id);

    assert_eq!(data.staged_file_exists()?, false);
    assert_eq!(data.committed_file_exists()?, true);
    assert_eq!(are_equal_files(src, &data.get_committed_file_path())?, true);

    // Test if document is updated correctly
    {
        let mut document = arhiv.get_document(&document.id)?.unwrap();
        document.data = json!({ "test": "other" });
        arhiv.stage_document(document, vec![])?;
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
    let prime = Arc::new(new_prime());
    let (join_handle, shutdown_sender, addr) = start_server(prime.clone());
    let replica = new_replica_with_port(addr.port());

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = new_document();
    document.refs.insert(attachment.id.clone());
    replica.stage_document(document.clone(), vec![attachment.clone()])?;

    replica.sync().await?;

    assert_eq!(
        replica.get_document(&document.id)?.unwrap().rev.is_staged(),
        false
    );

    // Test attachment data
    {
        let data = replica.get_attachment_data(&attachment.id);

        assert_eq!(data.staged_file_exists()?, false);
        assert_eq!(data.committed_file_exists()?, true);
        assert_eq!(are_equal_files(src, &data.get_committed_file_path())?, true);
    }

    {
        let data = prime.get_attachment_data(&attachment.id);

        assert_eq!(data.staged_file_exists()?, false);
        assert_eq!(data.committed_file_exists()?, true);
        assert_eq!(are_equal_files(src, &data.get_committed_file_path())?, true);
    }

    // Test if document is updated correctly
    {
        let mut document = replica.get_document(&document.id)?.unwrap();
        document.data = json!({ "test": "1" });
        replica.stage_document(document, vec![])?;
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
async fn test_download_attachment() -> Result<()> {
    let prime = Arc::new(new_prime());

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = new_document();
    document.refs.insert(attachment.id.clone());
    prime.stage_document(document, vec![attachment.clone()])?;

    prime.sync().await?;

    let (join_handle, shutdown_sender, addr) = start_server(prime.clone());
    let replica = new_replica_with_port(addr.port());

    replica.sync().await?;

    let data = replica.get_attachment_data(&attachment.id);
    data.download_data().await?;

    let dst = &data.get_committed_file_path();

    assert_eq!(are_equal_files(src, dst)?, true);

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}
