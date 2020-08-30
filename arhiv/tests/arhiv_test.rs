use anyhow::*;
use arhiv::utils::project_relpath;
use arhiv::{start_server, Arhiv, ArhivNotes};
use utils::*;

mod utils;

fn test_crud(arhiv: &Arhiv) -> Result<()> {
    // CREATE
    let mut document = ArhivNotes::create_note();
    document.data = ArhivNotes::data("test", "test");
    arhiv.stage_document(document.clone())?;
    assert_eq!(arhiv.list_documents(None)?.len(), 1);

    // READ
    {
        let other_document = arhiv.get_document(&document.id)?.unwrap();

        assert_eq!(other_document.data, document.data);
        assert_eq!(other_document.is_staged(), true);
    }

    // UPDATE
    {
        let mut other_document = arhiv.get_document(&document.id)?.unwrap();
        other_document.data = ArhivNotes::data("1", "1");
        arhiv.stage_document(other_document.clone())?;

        assert_eq!(
            arhiv.get_document(&document.id)?.unwrap().data,
            other_document.data
        );
    }

    // DELETE
    {
        assert_eq!(arhiv.list_documents(None)?.len(), 1);
        let mut other_document = arhiv.get_document(&document.id)?.unwrap();
        other_document.archived = true;
        arhiv.stage_document(other_document)?;

        assert_eq!(arhiv.get_document(&document.id)?.unwrap().archived, true);
        assert_eq!(arhiv.list_documents(None)?.len(), 0);
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

fn test_attachments(arhiv: &Arhiv) -> Result<()> {
    assert_eq!(arhiv.list_attachments(None)?.len(), 0);

    let src = &project_relpath("../resources/k2.jpg");
    let attachment = arhiv.stage_attachment(src, true)?;
    assert_eq!(
        arhiv
            .get_attachment_data(&attachment.id)
            .staged_file_exists()?,
        true
    );

    let dst = &arhiv
        .get_attachment_data(&attachment.id)
        .get_staged_file_path();

    assert_eq!(arhiv.list_attachments(None)?.len(), 1);
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
    let attachment = arhiv.stage_attachment(src, true)?;
    assert_eq!(attachment.filename, "k2.jpg");

    arhiv.update_attachment_filename(&attachment.id, "k1.jpg")?;
    assert_eq!(
        arhiv.get_attachment(&attachment.id)?.unwrap().filename,
        "k1.jpg"
    );

    assert_eq!(arhiv.get_status()?.rev, 0);

    let mut document = ArhivNotes::create_note();
    document.attachment_refs.push(attachment.id.clone());
    arhiv.stage_document(document.clone())?;

    arhiv.sync().await?;

    assert_eq!(arhiv.get_status()?.rev, 1);

    // make sure we increase rev after updating committed filename
    arhiv.update_attachment_filename(&attachment.id, "k1.jpg")?;
    assert_eq!(
        arhiv.get_attachment(&attachment.id)?.unwrap().filename,
        "k1.jpg"
    );
    assert_eq!(arhiv.get_status()?.rev, 2);

    Ok(())
}

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");
    let attachment = arhiv.stage_attachment(src, true)?;
    let other_attachment = arhiv.stage_attachment(src, true)?;

    let mut document = ArhivNotes::create_note();
    document.attachment_refs.push(attachment.id.clone());
    arhiv.stage_document(document.clone())?;
    assert_eq!(arhiv.get_document(&document.id)?.unwrap().is_staged(), true);

    assert_eq!(
        arhiv
            .get_attachment(&other_attachment.id)?
            .unwrap()
            .is_staged(),
        true
    );

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().is_staged(),
        false
    );
    assert_eq!(
        arhiv.get_attachment(&attachment.id)?.unwrap().is_staged(),
        false
    );

    // make sure unused attachment wasn't committed
    assert_eq!(
        arhiv
            .get_attachment(&other_attachment.id)?
            .unwrap()
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
        document.data = ArhivNotes::data("test", "test");
        arhiv.stage_document(document)?;
    }

    arhiv.sync().await?;

    assert_eq!(
        arhiv.get_document(&document.id)?.unwrap().data,
        ArhivNotes::data("test", "test")
    );

    Ok(())
}

#[tokio::test]
async fn test_replica_sync() -> Result<()> {
    let (prime, replica) = new_arhiv_pair();

    let (join_handle, shutdown_sender) = start_server(prime.clone());

    let src = &project_relpath("../resources/k2.jpg");
    let attachment = replica.stage_attachment(src, true)?;

    let mut document = ArhivNotes::create_note();
    document.attachment_refs.push(attachment.id.clone());
    replica.stage_document(document.clone())?;

    replica.sync().await?;

    assert_eq!(
        replica.get_document(&document.id)?.unwrap().is_staged(),
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
        document.data = ArhivNotes::data("test", "test");
        replica.stage_document(document)?;
    }

    replica.sync().await?;

    assert_eq!(
        replica.get_document(&document.id)?.unwrap().data,
        ArhivNotes::data("test", "test")
    );

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_download_attachment() -> Result<()> {
    let (prime, replica) = new_arhiv_pair();

    let src = &project_relpath("../resources/k2.jpg");
    let attachment = prime.stage_attachment(src, true)?;

    let mut document = ArhivNotes::create_note();
    document.attachment_refs.push(attachment.id.clone());
    prime.stage_document(document)?;

    prime.sync().await?;

    let (join_handle, shutdown_sender) = start_server(prime.clone());

    replica.sync().await?;

    let data = replica.get_attachment_data(&attachment.id);
    data.download_data().await?;

    let dst = &data.get_committed_file_path();

    assert_eq!(are_equal_files(src, dst)?, true);

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}
