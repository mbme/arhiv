use anyhow::Result;
use serde_json::json;

use rs_utils::http_server::Url;

use crate::{
    app::{App, AppResponse},
    components::Catalog,
    template_fn,
};

template_fn!(render_template, "./pick_document_modal.html.tera");

impl App {
    pub fn pick_document_modal(&self, url: Url) -> Result<AppResponse> {
        let catalog = Catalog::new(url).picker_mode().render(&self.arhiv)?;

        let content = render_template(json!({
            "catalog": catalog,
        }))?;

        Ok(AppResponse::fragment(content))
    }
}
