use anyhow::Result;

use rs_utils::{workspace_relpath, TempFile};

use super::utils::*;
use crate::{
    prime_server::{start_prime_server, PrimeServerRPC},
    test_arhiv::TestArhiv,
};

#[tokio::test]
async fn test_blobs() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let src = &workspace_relpath("resources/k2.jpg");

    let blob_id = arhiv.add_blob(src, false)?;

    assert!(arhiv.get_blob(&blob_id)?.exists()?);

    let mut document = empty_document();
    document.data.set("blob", &blob_id);

    arhiv.stage_document(&mut document)?;
    assert!(arhiv.get_blob(&blob_id)?.exists()?);

    // delete
    arhiv.erase_document(&document.id)?;
    arhiv.sync().await?;

    assert!(!arhiv.get_blob(&blob_id)?.exists()?);

    Ok(())
}

#[tokio::test]
async fn test_download_blob() -> Result<()> {
    let prime = TestArhiv::new_prime();

    let src = &workspace_relpath("resources/k2.jpg");

    let blob_id = prime.add_blob(src, false)?;

    let mut document = empty_document();
    document.data.set("blob", &blob_id);
    prime.stage_document(&mut document)?;

    prime.sync().await?;

    let (join_handle, shutdown_sender, addr) = start_prime_server(prime.0.clone(), 0);
    let replica = TestArhiv::new_replica(addr.port());

    replica.sync().await?;

    let blob = replica.get_blob(&blob_id)?;
    let prime_rpc = PrimeServerRPC::new(&replica.get_config().prime_url)?;
    prime_rpc.download_blob(&blob).await?;

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
    let resource_file_link = format!("{}/resource_file_link", temp_dir);
    std::os::unix::fs::symlink(&resource_file, &resource_file_link)?;

    {
        let result = arhiv.add_blob(&resource_file_link, false);
        assert!(result.is_err());
    }

    let resource_dir = workspace_relpath("resources");
    let resource_dir_link = format!("{}/resource_dir_link", &temp_dir);
    std::os::unix::fs::symlink(&resource_dir, &resource_dir_link)?;

    {
        let result = arhiv.add_blob(&resource_dir, false);
        assert!(result.is_err());
    }
    {
        let result = arhiv.add_blob(&resource_dir_link, false);
        assert!(result.is_err());
    }

    Ok(())
}
