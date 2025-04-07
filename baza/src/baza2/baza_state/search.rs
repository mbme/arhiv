use std::collections::HashMap;

use anyhow::Result;

use rs_utils::full_text_search::FTSEngine;

use crate::{
    entities::{Document, Id},
    schema::DataSchema,
    DocumentExpert,
};

pub struct SearchEngine {
    fts: FTSEngine,
    schema: DataSchema,
}

impl SearchEngine {
    pub fn new(schema: DataSchema) -> Self {
        SearchEngine {
            fts: FTSEngine::new(),
            schema,
        }
    }

    pub fn index_document(&mut self, document: &Document) -> Result<()> {
        let mut fields = HashMap::new();

        let document_expert = DocumentExpert::new(&self.schema);
        let title = document_expert.get_title(&document.document_type, &document.data)?;
        fields.insert("title", title.as_str());

        for field in self.schema.iter_fields(&document.document_type)? {
            let value = if let Some(value) = document.data.get(field.name) {
                value
            } else {
                continue;
            };

            let search_data = if let Some(search_data) = field.extract_search_data(value)? {
                search_data
            } else {
                continue;
            };

            fields.insert(field.name, search_data);
        }

        self.fts.index_document(document.id.to_string(), fields);

        Ok(())
    }

    pub fn remove_document_index(&mut self, id: &Id) {
        self.fts.remove_document(id);
    }

    pub fn search(&self, query: &str) -> impl Iterator<Item = Id> {
        let ids = self.fts.search(query);

        ids.into_iter().map(|id| id.into())
    }
}
