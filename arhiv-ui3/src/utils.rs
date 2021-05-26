use arhiv::{markup::RenderOptions, schema::SCHEMA};
use serde::Serialize;

const IGNORED_DOCUMENT_TYPES: &[&'static str] = &["tombstone", "attachment", "task"];

pub fn get_nav_document_types() -> Vec<&'static str> {
    SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .collect()
}

#[derive(Serialize, Clone)]
pub struct TemplateContext {
    pub markup_render_options: RenderOptions,
}

impl TemplateContext {
    pub fn new() -> Self {
        TemplateContext {
            markup_render_options: RenderOptions {
                document_path: "/documents".to_string(),
                attachment_data_path: "/attachment-data".to_string(),
            },
        }
    }
}
