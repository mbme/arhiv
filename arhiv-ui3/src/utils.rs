use arhiv::{
    entities::Id,
    markup::{MarkupRenderer, RenderOptions},
    schema::SCHEMA,
    Arhiv,
};

const IGNORED_DOCUMENT_TYPES: &[&'static str] = &["tombstone", "attachment", "task"];

pub fn get_nav_document_types() -> Vec<&'static str> {
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
}

impl AppContext {
    pub fn new(render_options: RenderOptions) -> Self {
        let arhiv = Arhiv::must_open();

        AppContext {
            arhiv,
            render_options,
        }
    }

    pub fn get_renderer(&self) -> MarkupRenderer {
        MarkupRenderer::new(&self.arhiv, &self.render_options)
    }

    pub fn get_document_url(&self, id: &Id) -> String {
        format!("{}/{}", self.render_options.document_path, id)
    }
}
