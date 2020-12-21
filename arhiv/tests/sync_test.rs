use anyhow::*;
use arhiv::entities::*;
use arhiv::start_server;
use rs_utils::project_relpath;
use serde_json::json;
use std::sync::Arc;
pub use utils::*;

mod utils;

#[tokio::test]
async fn test_prime_sync() -> Result<()> {
    let arhiv = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut other_attachment = AttachmentSource::new(src);
    other_attachment.copy = true;

    let mut document = empty_document();
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
