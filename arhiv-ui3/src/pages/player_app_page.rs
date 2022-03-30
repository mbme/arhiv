use anyhow::Result;
use serde_json::json;

use arhiv_core::{definitions::TRACK_TYPE, Filter};

use crate::{
    app::{App, AppResponse},
    template_fn,
    urls::document_url,
};

template_fn!(render_template, "./player_app_page.html.tera");

impl App {
    pub fn player_app_page(&self) -> Result<AppResponse> {
        let filter = Filter::default().with_document_type(TRACK_TYPE).all_items();

        let tracks = self
            .arhiv
            .list_documents(&filter)?
            .items
            .into_iter()
            .map(|document| {
                json!({
                    "url": document_url(&document.id, &None),
                    "artist": document.data.get_str("artist"),
                    "title": document.data.get_str("title"),
                    "track_id": document.data.get_str("track"),
                })
            })
            .collect::<Vec<_>>();

        let content = render_template(json!({
            "tracks": tracks,
        }))?;

        Ok(AppResponse::page("Player".to_string(), content))
    }
}
