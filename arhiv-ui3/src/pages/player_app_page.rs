use std::collections::HashSet;

use anyhow::Result;
use serde_json::json;

use arhiv_core::{
    definitions::{Attachment, TrackDocument, TRACK_TYPE},
    entities::Id,
    Filter,
};

use crate::{
    app::{App, AppResponse},
    template_fn,
    urls::document_url,
};

template_fn!(render_template, "./player_app_page.html.tera");

impl App {
    pub fn player_app_page(&self) -> Result<AppResponse> {
        let filter = Filter::default().with_document_type(TRACK_TYPE).all_items();

        let tx = self.arhiv.get_tx()?;

        let tracks: Vec<TrackDocument> = tx
            .list_documents(&filter)?
            .items
            .into_iter()
            .map(|document| document.try_into().expect("must be track"))
            .collect();

        let attachment_ids: HashSet<&Id> = tracks.iter().map(|track| &track.data.track).collect();

        let attachments: Vec<Attachment> = tx
            .get_documents(&attachment_ids)?
            .into_iter()
            .map(|document| document.try_into().expect("must be attachment"))
            .collect();

        let tracks = tracks
            .into_iter()
            .map(|track| {
                let attachment = attachments
                    .iter()
                    .find(|attachment| attachment.id == track.data.track)
                    .expect("coudn't find track attachment");

                json!({
                    "url": document_url(&track.id, &None),
                    "artist": track.data.artist,
                    "title": track.data.title,
                    "blob_id": attachment.data.blob,
                })
            })
            .collect::<Vec<_>>();

        let content = render_template(json!({
            "tracks": tracks,
        }))?;

        Ok(AppResponse::page("Player".to_string(), content))
    }
}
