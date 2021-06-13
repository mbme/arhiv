use anyhow::*;

use arhiv_core::{
    entities::Document,
    markup::{MarkupRenderer, MarkupStr, RenderOptions},
    schema::SCHEMA,
    Arhiv,
};

pub struct AppContext {
    pub arhiv: Arhiv,
    render_options: RenderOptions,
    pub document_types: Vec<&'static str>,
}

impl AppContext {
    pub fn new(render_options: RenderOptions) -> Result<Self> {
        let arhiv = Arhiv::open()?;

        let document_types: Vec<&str> = SCHEMA
            .modules
            .iter()
            .map(|module| module.document_type)
            .collect();

        Ok(AppContext {
            arhiv,
            render_options,
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
}
