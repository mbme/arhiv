use std::{collections::HashMap, io::Write, time::Instant};

use anyhow::{Context, Result};

use rs_utils::{
    AgeGzReader, AgeGzWriter,
    age::AgeKey,
    create_file_reader, create_file_writer,
    full_text_search::{FTSEngine, FieldBoost},
    log, read_all,
};

use crate::{
    DocumentExpert,
    entities::{Document, Id},
    schema::DataSchema,
};

const TITLE_FIELD_NAME: &str = "@title";
const ID_FIELD_NAME: &str = "@id";

pub struct SearchEngine {
    fts: FTSEngine,
    schema: DataSchema,
    modified: bool,
}

impl SearchEngine {
    pub fn new(schema: DataSchema) -> Self {
        SearchEngine {
            fts: FTSEngine::new(),
            schema,
            modified: false,
        }
    }

    pub fn read(file: &str, key: AgeKey, schema: DataSchema) -> Result<Self> {
        log::debug!("Reading search index from file {file}");

        let start_time = Instant::now();

        let reader = create_file_reader(file)?;
        let agegz_reader = AgeGzReader::new(reader, key)?;

        let bytes = read_all(agegz_reader)?;
        let fts: FTSEngine = postcard::from_bytes(&bytes).context("Failed to parse FTSEngine")?;

        let duration = start_time.elapsed();
        log::info!("Read search index from file in {:?}", duration);

        Ok(SearchEngine {
            fts,
            schema,
            modified: false,
        })
    }

    pub fn write(&mut self, file: &str, key: AgeKey) -> Result<()> {
        log::debug!("Writing search index to file {file}");

        let start_time = Instant::now();

        let writer = create_file_writer(file, true)?;
        let mut agegz_writer = AgeGzWriter::new(writer, key)?;

        postcard::to_io(&self.fts, &mut agegz_writer).context("Failed to serialize FTSEngine")?;

        let mut writer = agegz_writer.finish()?;
        writer.flush()?;

        self.modified = false;

        let duration = start_time.elapsed();
        log::info!("Wrote search index to file in {:?}", duration);

        Ok(())
    }

    pub fn index_document(&mut self, document: &Document) -> Result<()> {
        let mut fields = HashMap::new();

        let document_expert = DocumentExpert::new(&self.schema);
        let title = document_expert.get_title(&document.document_type, &document.data)?;
        fields.insert(TITLE_FIELD_NAME, title.as_str());
        fields.insert(ID_FIELD_NAME, &document.id);

        let mut boost_fields = HashMap::new();
        boost_fields.insert(TITLE_FIELD_NAME, FieldBoost::new(1.9)?);
        boost_fields.insert(ID_FIELD_NAME, FieldBoost::new(2.0)?);

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

        self.fts
            .index_document(document.id.to_string(), fields, boost_fields);

        self.modified = true;

        Ok(())
    }

    pub fn remove_document_index(&mut self, id: &Id) {
        self.fts.remove_document(id);

        self.modified = true;
    }

    pub fn search(&self, query: &str) -> impl Iterator<Item = Id> {
        let ids = self.fts.search(query);

        ids.into_iter().map(|id| id.into())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
