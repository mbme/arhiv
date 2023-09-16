use anyhow::Result;

use rs_utils::workspace_relpath;

use crate::{
    create_attachment,
    definitions::{get_standard_schema, Attachment},
    test_arhiv::TestArhiv,
};

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
