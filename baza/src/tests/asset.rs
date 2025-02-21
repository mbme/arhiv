use anyhow::Result;

use rs_utils::workspace_relpath;

use crate::{
    schema::{create_asset, Asset},
    Baza,
};

#[test]
fn test_create_asset() -> Result<()> {
    let baza = Baza::new_test_baza();

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = baza.get_tx()?;
    let asset = create_asset(&mut tx, src, false, None)?;
    tx.commit()?;

    assert!(baza.get_blob(&asset.data.blob)?.exists()?);
    assert!(baza.get_document(asset.id)?.is_some());

    Ok(())
}

#[test]
fn test_create_asset_with_custom_filename() -> Result<()> {
    let baza = Baza::new_test_baza();

    let src = &workspace_relpath("resources/k2.jpg");

    let mut tx = baza.get_tx()?;
    let asset = create_asset(&mut tx, src, false, Some("newname.jpg".to_string()))?;
    tx.commit()?;

    let asset: Asset = baza.get_document(asset.id)?.unwrap().convert()?;
    assert!(asset.data.filename == "newname.jpg");

    Ok(())
}
