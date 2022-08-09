use anyhow::Result;
use serde_json::json;

use crate::{
    app::{App, AppResponse},
    template_fn,
};

template_fn!(render_template, "./workspace_page.html.tera");

impl App {
    pub fn workspace_page(&self) -> Result<AppResponse> {
        let content = render_template(json!({}))?;

        Ok(AppResponse::fragment(content))
    }
}
