use crate::components::CatalogConfig;

pub fn get_catalog_config(document_type: impl AsRef<str>) -> CatalogConfig {
    let document_type = document_type.as_ref();

    if document_type == "book" {
        return CatalogConfig {
            fields: vec!["authors"],
            preview: None,
        };
    }

    if document_type == "task" {
        return CatalogConfig {
            fields: vec!["status"],
            preview: None,
        };
    }

    CatalogConfig::default()
}
