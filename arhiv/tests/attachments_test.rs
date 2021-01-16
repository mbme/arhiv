use anyhow::*;
use arhiv::{entities::*, start_server, Filter, Matcher};
use rs_utils::project_relpath;
use std::sync::Arc;
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
            .get_attachment_data(&attachment.id)
            .staged_file_exists()?,
        true
    );

    let dst = &arhiv
        .get_attachment_data(&attachment.id)
        .get_staged_file_path();

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
    let prime = Arc::new(new_prime());

    let src = &project_relpath("../resources/k2.jpg");

    let mut attachment = AttachmentSource::new(src);
    attachment.copy = true;

    let mut document = empty_document();
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
