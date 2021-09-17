use crate::components::CatalogConfig;

pub struct UIConfig {
    pub catalog: CatalogConfig,
}

impl UIConfig {
    pub fn get_config(document_type: impl AsRef<str>) -> Self {
        let document_type = document_type.as_ref();

        if document_type == "book" {
            return UIConfig {
                catalog: CatalogConfig {
                    fields: vec!["authors"],
                    ..CatalogConfig::default()
                },
            };
        }

        UIConfig {
            catalog: CatalogConfig::default(),
        }
    }

    pub fn get_child_config(
        parent_document_type: impl AsRef<str>,
        child_document_type: impl AsRef<str>,
    ) -> Self {
        UIConfig::get_config(format!(
            "{}/{}",
            parent_document_type.as_ref(),
            child_document_type.as_ref()
        ))
    }
}
