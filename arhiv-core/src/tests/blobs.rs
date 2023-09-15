use anyhow::Result;

use rs_utils::{workspace_relpath, TempFile};

use super::utils::*;
use crate::{
    create_attachment,
    definitions::{get_standard_schema, Attachment},
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
