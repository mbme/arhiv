use anyhow::{bail, ensure, Context, Result};

use arhiv_core::{
    definitions::{Attachment, ATTACHMENT_TYPE, TRACK_TYPE},
    entities::{Document, DocumentData},
    Arhiv,
};
use rs_utils::{ensure_file_exists, remove_file_extension};

pub fn import_document_from_file(
    arhiv: &Arhiv,
    document_type: &str,
    file_path: &str,
    move_file: bool,
) -> Result<Document> {
    ensure_file_exists(file_path)?;

    match document_type {
        TRACK_TYPE => import_track(arhiv, file_path, move_file),
        ATTACHMENT_TYPE => {
            let attachment = Attachment::create(file_path, move_file, arhiv)?;

            Ok(attachment.into())
        }
        other => bail!("Don't know how to import document of type '{}'", other),
    }
}

fn import_track(arhiv: &Arhiv, file_path: &str, move_file: bool) -> Result<Document> {
    let mut tx = arhiv.get_tx()?;

    let attachment = Attachment::create_tx(file_path, move_file, &mut tx)?;

    ensure!(
        attachment.is_audio(),
        "file type must be audio, got {}",
        attachment.get_media_type()
    );

    let file_name = remove_file_extension(attachment.get_filename())?;

    let mut iter = file_name.split('-');

    let artist = iter
        .next()
        .context("couldn't extract track artist from filename")?;

    let title = iter
        .next()
        .context("couldn't extract track title from filename")?;

    let mut data = DocumentData::new();
    data.set("artist", artist);
    data.set("title", title);
    data.set("track", &attachment.id);

    let mut document = Document::new_with_data(TRACK_TYPE, data);

    tx.stage_document(&mut document)?;

    tx.commit()?;

    Ok(document)
}
