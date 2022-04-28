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

fn format_duration(duration_ms: u64) -> String {
    let duration_s = (duration_ms as f64 / 1000.0).round() as u64;

    let seconds = duration_s % 60;
    let minutes = (duration_s / 60) % 60;
    let hours = (duration_s / 60) / 60;

    if hours > 0 {
        format!("{}:{:0>2}:{:0>2}", hours, minutes, seconds)
    } else {
        format!("{}:{:0>2}", minutes, seconds)
    }
}

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

        let total_duration_ms: u64 = attachments
            .iter()
            .map(|attachment| attachment.data.duration.unwrap_or_default())
            .sum();

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
                    "duration": format_duration(attachment.data.duration.unwrap_or_default()),
                })
            })
            .collect::<Vec<_>>();

        let content = render_template(json!({
            "tracks": tracks,
            "total_duration": format_duration(total_duration_ms),
        }))?;

        Ok(AppResponse::page("Player".to_string(), content))
    }
}
