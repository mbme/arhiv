use anyhow::*;
use arhiv::entities::*;
use arhiv::start_server;
use rs_utils::project_relpath;
use serde_json::json;
pub use utils::*;

mod utils;

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());

    arhiv.stage_document(document.clone(), vec![attachment.clone()])?;

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
    let prime = new_prime();
    let (join_handle, shutdown_sender, addr) = start_server(prime.unwrap());
    let replica = new_replica_with_port(addr.port());

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = empty_document();
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
