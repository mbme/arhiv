use anyhow::{Context, Result};
use serde_json::json;

use crate::{
    app::{App, AppResponse},
    template_fn,
};

template_fn!(render_template, "./workspace_page.html.tera");

impl App {
    pub fn workspace_page(&self) -> Result<AppResponse> {
        let schema = serde_json::to_string_pretty(self.arhiv.get_schema())
            .context("failed to serialize schema")?;

        let content = render_template(json!({
            "schema": schema,
        }))?;

        Ok(AppResponse::fragment(content))
    }
}
