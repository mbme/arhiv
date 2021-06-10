use anyhow::*;
use hyper::{header, Response};
use serde::Serialize;
use serde_json::{json, Value};

use arhiv::{
    entities::Document,
    markup::{MarkupRenderer, MarkupStr, RenderOptions},
    schema::SCHEMA,
    Arhiv,
};
use templates::Templates;

use crate::http_utils::AppResponse;

mod templates;

const IGNORED_DOCUMENT_TYPES: &[&'static str] = &["tombstone", "attachment", "task"];

fn get_nav_document_types() -> Vec<&'static str> {
    SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .collect()
}

pub struct AppContext {
    pub arhiv: Arhiv,
    render_options: RenderOptions,
    templates: Templates,
    pub document_types: Vec<&'static str>,
}

impl AppContext {
    pub fn new(render_options: RenderOptions) -> Result<Self> {
        let arhiv = Arhiv::open()?;

        let global_context = json!({ "nav_document_types": get_nav_document_types() });

        let document_types: Vec<&str> = SCHEMA
            .modules
            .iter()
            .map(|module| module.document_type)
            .collect();

        Ok(AppContext {
            arhiv,
            render_options,
            templates: Templates::new(global_context)?,
            document_types,
        })
    }

    pub fn render_markup(&self, string: &MarkupStr) -> String {
        let renderer = MarkupRenderer::new(&self.arhiv, &self.render_options);

        renderer.to_html(string)
    }

    pub fn render_preview(&self, document: &Document) -> String {
        let renderer = MarkupRenderer::new(&self.arhiv, &self.render_options);

        renderer
            .get_preview(document)
            .unwrap_or("Unable to generate preview".to_string())
    }

    pub fn render_page(&self, template_name: &str, context: Value) -> AppResponse {
        let result = self.templates.render(template_name, context)?;

        Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            // prevent page from caching
            .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
            .header(header::EXPIRES, "0")
            // ---
            .body(result.into())
            .context("failed to build response")
    }

    pub fn render_template(&self, template_name: &str, context: impl Serialize) -> Result<String> {
        self.templates
            .render(template_name, serde_json::to_value(context)?)
    }
}
