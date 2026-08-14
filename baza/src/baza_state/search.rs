use std::{collections::HashMap, io::Write, time::Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::full_text_search::{FTSEngine, FieldBoost};
use baza_common::{create_file_reader, create_file_writer, log, read_all};
use baza_storage::crypto::age::AgeKey;
use baza_storage::{AgeGzReader, AgeGzWriter};

use crate::{
    DocumentExpert,
    entities::{Document, Id},
    schema::DataSchema,
};

const TITLE_FIELD_NAME: &str = "@title";
const ID_FIELD_NAME: &str = "@id";
const SEARCH_INDEX_FORMAT_VERSION: u8 = 1;
// v5 uses field-local BM25 document-length normalization.
const SEARCH_ALGORITHM_VERSION: u8 = 5;

#[derive(Serialize, Deserialize)]
struct SearchIndexFile {
    format_version: u8,
    search_version: u8,
    data_version: u8,
    schema_fingerprint: String,
    fts: FTSEngine,
}

impl SearchIndexFile {
    fn validate(&self, schema: &DataSchema) -> Result<()> {
        if self.format_version != SEARCH_INDEX_FORMAT_VERSION {
            log::info!(
                "Search index format version mismatch: expected {}, found {}",
                SEARCH_INDEX_FORMAT_VERSION,
                self.format_version
            );
            bail!("Search index format version mismatch");
        }

        if self.search_version != SEARCH_ALGORITHM_VERSION {
            log::info!(
                "Search index algorithm version mismatch: expected {}, found {}",
                SEARCH_ALGORITHM_VERSION,
                self.search_version
            );
            bail!("Search index algorithm version mismatch");
        }

        let expected_data_version = schema.get_latest_data_version();
        if self.data_version != expected_data_version {
            log::info!(
                "Search index data version mismatch: expected {}, found {}",
                expected_data_version,
                self.data_version
            );
            bail!("Search index data version mismatch");
        }

        let expected_schema_fingerprint = schema.fingerprint()?;
        if self.schema_fingerprint != expected_schema_fingerprint {
            log::info!(
                "Search index schema fingerprint mismatch: expected {}, found {}",
                expected_schema_fingerprint,
                self.schema_fingerprint
            );
            bail!("Search index schema fingerprint mismatch");
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct SearchIndexFileRef<'fts> {
    format_version: u8,
    search_version: u8,
    data_version: u8,
    schema_fingerprint: String,
    fts: &'fts FTSEngine,
}

impl<'fts> SearchIndexFileRef<'fts> {
    fn new(schema: &DataSchema, fts: &'fts FTSEngine) -> Result<Self> {
        Ok(Self {
            format_version: SEARCH_INDEX_FORMAT_VERSION,
            search_version: SEARCH_ALGORITHM_VERSION,
            data_version: schema.get_latest_data_version(),
            schema_fingerprint: schema.fingerprint()?,
            fts,
        })
    }
}

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
        let index_file: SearchIndexFile =
            postcard::from_bytes(&bytes).context("Failed to parse SearchIndexFile")?;
        index_file.validate(&schema)?;

        let duration = start_time.elapsed();
        log::info!(
            "Read search index from file in {:?}: format_version={}, search_version={}, data_version={}, schema_fingerprint={}",
            duration,
            index_file.format_version,
            index_file.search_version,
            index_file.data_version,
            index_file.schema_fingerprint
        );

        Ok(SearchEngine {
            fts: index_file.fts,
            schema,
            modified: false,
        })
    }

    pub fn write(&mut self, file: &str, key: AgeKey) -> Result<()> {
        log::debug!("Writing search index to file {file}");

        let start_time = Instant::now();

        let writer = create_file_writer(file, true)?;
        let mut agegz_writer = AgeGzWriter::new(writer, key)?;

        let index_file = SearchIndexFileRef::new(&self.schema, &self.fts)?;
        postcard::to_io(&index_file, &mut agegz_writer)
            .context("Failed to serialize SearchIndexFile")?;

        let mut writer = agegz_writer.finish()?;
        writer.flush()?;

        self.modified = false;

        let duration = start_time.elapsed();
        log::info!(
            "Wrote search index to file in {:?}: format_version={}, search_version={}, data_version={}, schema_fingerprint={}",
            duration,
            index_file.format_version,
            index_file.search_version,
            index_file.data_version,
            index_file.schema_fingerprint
        );

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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use baza_common::{TempFile, create_file_writer};
    use baza_storage::AgeGzWriter;
    use baza_storage::crypto::age::AgeKey;
    use serde_json::json;

    use crate::{
        entities::{Id, new_document},
        full_text_search::FTSEngine,
        schema::{DataDescription, DataSchema},
    };

    use super::{
        SEARCH_ALGORITHM_VERSION, SEARCH_INDEX_FORMAT_VERSION, SearchEngine, SearchIndexFile,
    };

    fn write_search_index_file(
        file: &str,
        key: AgeKey,
        schema: &DataSchema,
        format_version: u8,
        search_version: u8,
        data_version: u8,
    ) {
        let writer = create_file_writer(file, true).unwrap();
        let mut agegz_writer = AgeGzWriter::new(writer, key).unwrap();
        let index_file = SearchIndexFile {
            format_version,
            search_version,
            data_version,
            schema_fingerprint: schema.fingerprint().unwrap(),
            fts: FTSEngine::new(),
        };

        postcard::to_io(&index_file, &mut agegz_writer).unwrap();
        let mut writer = agegz_writer.finish().unwrap();
        writer.flush().unwrap();
    }

    #[test]
    fn test_indexes_document_id_and_searchable_fields() {
        let mut search = SearchEngine::new(DataSchema::new_test_schema());
        let document =
            new_document(json!({ "test": "searchable body" })).with_id(Id::from("knownsearchid"));

        search.index_document(&document).unwrap();

        assert_eq!(
            search.search("knownsearchid").collect::<Vec<_>>(),
            vec![document.id.clone()]
        );
        assert_eq!(
            search.search("searchable").collect::<Vec<_>>(),
            vec![document.id.clone()]
        );
    }

    #[test]
    fn test_ignores_non_searchable_ref_fields() {
        let mut search = SearchEngine::new(DataSchema::new_test_schema());
        let document = new_document(json!({
            "test": "owner",
            "ref": "referenceddoc",
        }))
        .with_id(Id::from("ownerdoc"));

        search.index_document(&document).unwrap();

        assert!(search.search("referenceddoc").next().is_none());
    }

    #[test]
    fn test_index_document_rejects_invalid_searchable_field_type() {
        let mut search = SearchEngine::new(DataSchema::new_test_schema());
        let document = new_document(json!({ "test": 123 }));

        let err = search.index_document(&document).unwrap_err();

        assert!(err.to_string().contains("failed to extract field test"));
    }

    #[test]
    fn test_search_index_read_write_roundtrip() {
        let schema = DataSchema::new_test_schema();
        let key = AgeKey::generate_age_x25519_key();
        let file = TempFile::new();
        let document = new_document(json!({ "test": "roundtrip searchable" }));

        let mut search = SearchEngine::new(schema.clone());
        search.index_document(&document).unwrap();
        search.write(&file.path, key.clone()).unwrap();

        let search = SearchEngine::read(&file.path, key, schema).unwrap();

        assert_eq!(
            search.search("roundtrip").collect::<Vec<_>>(),
            vec![document.id.clone()]
        );
    }

    #[test]
    fn test_search_index_read_rejects_schema_fingerprint_mismatch() {
        let key = AgeKey::generate_age_x25519_key();
        let file = TempFile::new();
        let document = new_document(json!({ "test": "schema mismatch" }));

        let mut search = SearchEngine::new(DataSchema::new_test_schema());
        search.index_document(&document).unwrap();
        search.write(&file.path, key.clone()).unwrap();

        let changed_schema = DataSchema::new(
            "test",
            vec![DataDescription {
                document_type: "different_type",
                title_format: "${test}",
                fields: vec![],
            }],
        );

        let err = match SearchEngine::read(&file.path, key, changed_schema) {
            Ok(_) => panic!("Search index read should reject schema fingerprint mismatch"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("schema fingerprint mismatch"));
    }

    #[test]
    fn test_search_index_read_rejects_format_version_mismatch() {
        let schema = DataSchema::new_test_schema();
        let key = AgeKey::generate_age_x25519_key();
        let file = TempFile::new();

        write_search_index_file(
            &file.path,
            key.clone(),
            &schema,
            SEARCH_INDEX_FORMAT_VERSION + 1,
            SEARCH_ALGORITHM_VERSION,
            schema.get_latest_data_version(),
        );

        let err = match SearchEngine::read(&file.path, key, schema) {
            Ok(_) => panic!("Search index read should reject format version mismatch"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("format version mismatch"));
    }

    #[test]
    fn test_search_index_read_rejects_search_algorithm_version_mismatch() {
        let schema = DataSchema::new_test_schema();
        let key = AgeKey::generate_age_x25519_key();
        let file = TempFile::new();

        write_search_index_file(
            &file.path,
            key.clone(),
            &schema,
            SEARCH_INDEX_FORMAT_VERSION,
            SEARCH_ALGORITHM_VERSION + 1,
            schema.get_latest_data_version(),
        );

        let err = match SearchEngine::read(&file.path, key, schema) {
            Ok(_) => panic!("Search index read should reject search algorithm version mismatch"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("algorithm version mismatch"));
    }

    #[test]
    fn test_search_index_read_rejects_data_version_mismatch() {
        let schema = DataSchema::new_test_schema();
        let key = AgeKey::generate_age_x25519_key();
        let file = TempFile::new();

        write_search_index_file(
            &file.path,
            key.clone(),
            &schema,
            SEARCH_INDEX_FORMAT_VERSION,
            SEARCH_ALGORITHM_VERSION,
            schema.get_latest_data_version() + 1,
        );

        let err = match SearchEngine::read(&file.path, key, schema) {
            Ok(_) => panic!("Search index read should reject data version mismatch"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("data version mismatch"));
    }
}
