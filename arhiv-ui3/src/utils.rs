use arhiv::schema::SCHEMA;

const IGNORED_DOCUMENT_TYPES: &[&'static str] = &["tombstone", "attachment", "task"];

fn get_nav_document_types() -> Vec<&'static str> {
    SCHEMA
        .modules
        .iter()
        .map(|module| module.document_type)
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .collect()
}

#[derive(Clone)]
pub struct TemplateContext {
    pub nav_document_types: Vec<&'static str>,
}

impl TemplateContext {
    pub fn new() -> Self {
        TemplateContext {
            nav_document_types: get_nav_document_types(),
        }
    }
}
