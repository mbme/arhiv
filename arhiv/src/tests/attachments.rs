use super::utils::*;
use crate::{entities::*, start_server, Filter, Matcher};
use anyhow::*;
use rs_utils::project_relpath;

#[test]
fn test_attachments() -> Result<()> {
    let arhiv = new_prime();
    assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 0);

    let src = &project_relpath("../resources/k2.jpg");

    let attachment = arhiv.add_attachment(src, true)?;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());

    arhiv.stage_document(document)?;
    assert_eq!(
        arhiv.get_attachment_data_by_id(&attachment.id)?.exists()?,
        true
    );

    let page = arhiv.list_documents(Filter {
        matchers: vec![Matcher::Type {
            document_type: ATTACHMENT_TYPE.to_string(),
        }],
        ..Filter::default()
    })?;
    assert_eq!(page.items.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_download_attachment() -> Result<()> {
    let prime = new_prime();

    let src = &project_relpath("../resources/k2.jpg");

    let attachment = prime.add_attachment(src, true)?;

    let mut document = empty_document();
    document.refs.insert(attachment.id.clone());
    prime.stage_document(document)?;

    prime.sync().await?;

    let (join_handle, shutdown_sender, addr) = start_server(prime);
    let replica = new_replica(addr.port());

    replica.sync().await?;

    let attachment_data = replica.get_attachment_data_by_id(&attachment.id)?;
    replica
        .get_network_service()?
        .download_attachment_data(&attachment_data)
        .await?;

    let dst = &attachment_data.path;

    assert_eq!(are_equal_files(src, dst)?, true);

    shutdown_sender.send(()).unwrap();
    join_handle.await.unwrap();

    Ok(())
}
