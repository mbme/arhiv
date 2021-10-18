use arhiv_core::definitions::{BOOK_TYPE, FILM_TYPE};

use crate::components::CatalogConfig;

pub fn get_catalog_config(document_type: impl AsRef<str>) -> CatalogConfig {
    let document_type = document_type.as_ref();

    if document_type == BOOK_TYPE {
        return CatalogConfig {
            fields: vec!["authors", "rating"],
            preview: None,
        };
    }

    if document_type == FILM_TYPE {
        return CatalogConfig {
            fields: vec!["duration", "release_date", "rating"],
            preview: None,
        };
    }

    CatalogConfig::default()
}
