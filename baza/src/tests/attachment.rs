use anyhow::Result;

use rs_utils::workspace_relpath;

use crate::{
    schema::{create_attachment, Attachment},
    Baza,
};

#[test]
fn test_create_attachment() -> Result<()> {
    let baza = Baza::new_test_baza();

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = baza.get_tx()?;
    let attachment = create_attachment(&mut tx, src, false, None)?;
    tx.commit()?;

    assert!(baza.get_blob(&attachment.data.blob)?.exists()?);
    assert!(baza.get_document(&attachment.id)?.is_some());

    Ok(())
}

#[test]
fn test_create_attachment_with_custom_filename() -> Result<()> {
    let baza = Baza::new_test_baza();

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = baza.get_tx()?;
    let attachment = create_attachment(&mut tx, src, false, Some("newname.jpg".to_string()))?;
    tx.commit()?;

    let attachment: Attachment = baza.get_document(attachment.id)?.unwrap().convert()?;
    assert!(attachment.data.filename == "newname.jpg");

    Ok(())
}
