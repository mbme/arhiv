mod temp_arhiv;

use anyhow::*;
use arhiv::ArhivNotes;
use temp_arhiv::TempArhiv;

#[test]
fn it_works() -> Result<()> {
    let arhiv = TempArhiv::new(true);
    assert_eq!(arhiv.list_documents(None)?.len(), 0);

    Ok(())
}

#[test]
fn add_document() -> Result<()> {
    let arhiv = TempArhiv::new(true);
    let mut document = ArhivNotes::create_note();
    document.data = ArhivNotes::data("test", "test");

    arhiv.stage_document(document.clone())?;
    assert_eq!(arhiv.list_documents(None)?.len(), 1);

    {
        let other_document = arhiv.get_document(&document.id)?.unwrap();

        assert_eq!(other_document.data, document.data);
        assert_eq!(other_document.is_staged(), true);
    }

    arhiv.sync()?;

    {
        let other_document = arhiv.get_document(&document.id)?.unwrap();

        assert_eq!(other_document.is_staged(), false);
    }

    Ok(())
}

#[test]
fn delete_document() -> Result<()> {
    let arhiv = TempArhiv::new(true);

    let mut document = ArhivNotes::create_note();
    document.archived = true;

    arhiv.stage_document(document.clone())?;
    assert_eq!(arhiv.list_documents(None)?.len(), 0);
    arhiv.sync()?;

    {
        let other_document = arhiv.get_document(&document.id)?.unwrap();
        assert_eq!(other_document.is_staged(), false);
        assert_eq!(other_document.archived, true);
    }

    Ok(())
}
