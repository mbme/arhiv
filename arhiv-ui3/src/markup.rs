use arhiv_core::{
    markup::{MarkupRenderer, MarkupStr, RenderOptions},
    Arhiv,
};

pub trait ArhivMarkupExt {
    fn get_renderer(&self) -> MarkupRenderer;

    fn render_markup(&self, string: &MarkupStr) -> String {
        let renderer = self.get_renderer();

        renderer.to_html(string)
    }

    fn render_preview(&self, text: &str) -> String {
        self.get_renderer().get_preview(text)
    }
}

impl ArhivMarkupExt for Arhiv {
    fn get_renderer(&self) -> MarkupRenderer {
        MarkupRenderer::new(
            &self,
            RenderOptions {
                document_path: "/documents".to_string(),
                attachment_data_path: "/attachment-data".to_string(),
            },
        )
    }
}
