use anyhow::Result;

use rs_utils::{workspace_relpath, TempFile};

use super::utils::*;
use crate::{
    create_attachment,
    definitions::{get_standard_schema, Attachment},
    prime_server::start_prime_server,
    test_arhiv::TestArhiv,
};

#[tokio::test]
async fn test_blobs() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let src = &workspace_relpath("resources/k2.jpg");

    let blob_id = {
        let mut tx = arhiv.baza.get_tx()?;

        let blob_id = tx.add_blob(src, false)?;
        tx.commit()?;

        blob_id
    };

    assert!(arhiv.baza.get_blob(&blob_id)?.exists()?);

    let mut document = empty_document();
    document.data.set("blob", &blob_id);

    {
        let tx = arhiv.baza.get_tx()?;
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }
    assert!(arhiv.baza.get_blob(&blob_id)?.exists()?);

    // delete
    {
        let tx = arhiv.baza.get_tx()?;
        tx.erase_document(&document.id)?;
        tx.commit()?;
    }

    arhiv.sync().await?;

    assert!(!arhiv.baza.get_blob(&blob_id)?.exists()?);

    Ok(())
}

#[tokio::test]
async fn test_download_blob_during_sync() -> Result<()> {
    let prime = TestArhiv::new_prime();

    let src = &workspace_relpath("resources/k2.jpg");

    let blob_id = {
        let mut tx = prime.baza.get_tx()?;

        let blob_id = tx.add_blob(src, false)?;

        let mut document = empty_document();
        document.data.set("blob", &blob_id);
        tx.stage_document(&mut document)?;

        tx.commit()?;

        blob_id
    };

    prime.sync().await?;

    let (join_handle, shutdown_sender, addr) = start_prime_server(prime.0.clone(), 0);
    let replica = TestArhiv::new_replica(addr.port());

    replica.sync().await?;

    let blob = replica.baza.get_blob(&blob_id)?;

    let dst = &blob.file_path;

    assert!(are_equal_files(src, dst)?);

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}

#[test]
fn test_add_blob_soft_links_and_dirs() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let temp_dir = TempFile::new_with_details("test_add_blob_soft_links_and_dirs", "");
    temp_dir.mkdir()?;

    let resource_file = workspace_relpath("resources/k2.jpg");
    let resource_file_link = format!("{temp_dir}/resource_file_link");
    std::os::unix::fs::symlink(resource_file, &resource_file_link)?;

    {
        let mut tx = arhiv.baza.get_tx()?;
        let result = tx.add_blob(&resource_file_link, false);
        assert!(result.is_err());
    }

    let resource_dir = workspace_relpath("resources");
    let resource_dir_link = format!("{}/resource_dir_link", &temp_dir);
    std::os::unix::fs::symlink(&resource_dir, &resource_dir_link)?;

    {
        let mut tx = arhiv.baza.get_tx()?;
        let result = tx.add_blob(&resource_dir, false);
        assert!(result.is_err());
    }

    {
        let mut tx = arhiv.baza.get_tx()?;
        let result = tx.add_blob(&resource_dir_link, false);
        assert!(result.is_err());
    }

    Ok(())
}

#[tokio::test]
async fn test_create_attachment() -> Result<()> {
    let arhiv = TestArhiv::new_prime_with_schema(get_standard_schema());

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = arhiv.baza.get_tx()?;
    let attachment = create_attachment(&mut tx, src, false, None)?;
    tx.commit()?;

    assert!(arhiv.baza.get_blob(&attachment.data.blob)?.exists()?);
    assert!(arhiv.baza.get_document(&attachment.id)?.is_some());

    Ok(())
}

#[tokio::test]
async fn test_create_attachment_with_custom_filename() -> Result<()> {
    let arhiv = TestArhiv::new_prime_with_schema(get_standard_schema());

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = arhiv.baza.get_tx()?;
    let attachment = create_attachment(&mut tx, src, false, Some("newname.jpg".to_string()))?;
    tx.commit()?;

    let attachment: Attachment = arhiv.baza.get_document(attachment.id)?.unwrap().convert()?;
    assert!(attachment.data.filename == "newname.jpg");

    Ok(())
}
