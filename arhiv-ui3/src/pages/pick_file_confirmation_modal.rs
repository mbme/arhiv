use std::fs;

use anyhow::{anyhow, Result};
use serde_json::json;

use rs_utils::{ensure_file_exists, server::Url};

use crate::{
    app::{App, AppResponse},
    template_fn,
    urls::pick_file_confirmation_handler_url,
};

template_fn!(render_template, "./pick_file_confirmation_modal.html.tera");

impl App {
    pub fn pick_file_confirmation_modal(url: &Url) -> Result<AppResponse> {
        let file_path = url
            .get_query_param("file")
            .ok_or_else(|| anyhow!("file query param must be present"))?;

        ensure_file_exists(file_path)?;

        let metadata = fs::metadata(file_path)?;
        let size = metadata.len();

        let content = render_template(json!({
            "handler_url": pick_file_confirmation_handler_url(),
            "file_path": file_path,
            "size": size,
        }))?;

        Ok(AppResponse::fragment(content))
    }
}
