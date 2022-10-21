use anyhow::{Context, Result};
use serde_json::json;
use tera::{Context as TeraContext, Tera};

use crate::include_dynamic_str;

use super::app::{App, AppResponse};

impl App {
    pub fn workspace_page(&self) -> Result<AppResponse> {
        let template = include_dynamic_str!("./workspace_page.html.tera")?;

        let schema = serde_json::to_string_pretty(self.arhiv.get_schema())
            .context("failed to serialize schema")?;

        let context = json!({
            "schema": schema,
        });
        let context = TeraContext::from_value(
            serde_json::to_value(context).context("failed to serialize context")?,
        )
        .context("failed to create context")?;

        let content =
            Tera::one_off(&template, &context, true).context("failed to render template")?;

        Ok(AppResponse::fragment(content))
    }
}
