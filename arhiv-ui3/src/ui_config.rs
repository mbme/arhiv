use crate::components::CatalogConfig;

pub fn get_catalog_config(document_type: impl AsRef<str>) -> CatalogConfig {
    let document_type = document_type.as_ref();

    if document_type == "book" {
        return CatalogConfig {
            fields: vec!["authors"],
            ..CatalogConfig::default()
        };
    }

    CatalogConfig::default()
}
