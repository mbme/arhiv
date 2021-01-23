use anyhow::*;
use arhiv::{entities::*, start_server, Filter, Matcher};
use rs_utils::project_relpath;
pub use utils::*;

mod utils;

#[test]
fn test_attachments() -> Result<()> {
    let arhiv = new_prime();
    assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 0);

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());

    arhiv.stage_document(document, vec![attachment.clone()])?;
    assert_eq!(
        arhiv
            .get_data_service()
            .staged_file_exists(&attachment.id)?,
        true
    );

    let dst = &arhiv
        .get_data_service()
        .get_staged_file_path(&attachment.id);

    let page = arhiv.list_documents(Filter {
        matchers: vec![Matcher::Type {
            document_type: ATTACHMENT_TYPE.to_string(),
        }],
        ..Filter::default()
    })?;
    assert_eq!(page.items.len(), 1);
    assert_eq!(are_equal_files(src, dst)?, true);

    Ok(())
}

#[tokio::test]
async fn test_download_attachment() -> Result<()> {
    let prime = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());
    prime.stage_document(document, vec![attachment.clone()])?;

    prime.sync().await?;

    let (join_handle, shutdown_sender, addr) = start_server(prime.unwrap());
    let replica = new_replica_with_port(addr.port());

    replica.sync().await?;

    replica
        .get_network_service()
        .download_attachment_data(&attachment.id)
        .await?;

    let dst = &replica
        .get_data_service()
        .get_committed_file_path(&attachment.id);

    assert_eq!(are_equal_files(src, dst)?, true);

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}
