use anyhow::*;

use rs_utils::project_relpath;

use super::utils::*;
use crate::{
    prime_server::{start_prime_server, PrimeServerRPC},
    Filter,
};

#[tokio::test]
async fn test_blobs() -> Result<()> {
    let arhiv = new_prime();
    assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 0);

    let src = &project_relpath("../resources/k2.jpg");

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
    let prime = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let blob_id = prime.add_blob(src, false)?;

    let mut document = empty_document();
    document.data.set("blob", &blob_id);
    prime.stage_document(&mut document)?;

    prime.sync().await?;

    let (join_handle, shutdown_sender, addr) = start_prime_server(prime, 0);
    let replica = new_replica(addr.port());

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
