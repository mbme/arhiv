use std::{fs, sync::Arc};

use anyhow::Result;
use serde_json::Value;

use rs_utils::generate_temp_path;

use crate::{
    definitions::get_standard_schema,
    entities::Document,
    schema::{Collection, DataDescription, Field, FieldType},
    Arhiv, Config, ListPage,
};

impl Drop for Arhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.get_config().arhiv_root)
            .expect("must be able to remove arhiv");
    }
}

fn new_arhiv(config: Config, prime: bool, data_description: DataDescription) -> Arc<Arhiv> {
    let mut schema = get_standard_schema();

    schema.with_modules(&mut vec![data_description]);

    let arhiv = Arhiv::create(config, schema, "test-arhiv".to_string(), prime)
        .expect("must be able to create temp arhiv");

    Arc::new(arhiv)
}

pub fn new_prime() -> Arc<Arhiv> {
    new_prime_with_schema(DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![
            Field {
                name: "blob",
                field_type: FieldType::BLOBId,
                mandatory: false,
                readonly: false,
            },
            Field {
                name: "test",
                field_type: FieldType::String {},
                mandatory: false,
                readonly: false,
            },
        ],
    })
}

pub fn new_prime_with_schema(data_description: DataDescription) -> Arc<Arhiv> {
    let config = Config {
        arhiv_root: generate_temp_path("TempArhiv", ""),
        ..Config::default()
    };

    new_arhiv(config, true, data_description)
}

pub fn new_replica(port: u16) -> Arc<Arhiv> {
    let config = Config {
        arhiv_root: generate_temp_path("TempArhiv", ""),
        prime_url: format!("http://localhost:{}", port),
        ..Config::default()
    };

    let data_description = DataDescription {
        document_type: "test_type",
        collection_of: Collection::None,
        fields: vec![
            Field {
                name: "blob",
                field_type: FieldType::BLOBId,
                mandatory: false,
                readonly: false,
            },
            Field {
                name: "test",
                field_type: FieldType::String {},
                mandatory: false,
                readonly: false,
            },
        ],
    };

    new_arhiv(config, false, data_description)
}

pub fn empty_document() -> Document {
    Document::new("test_type")
}

pub fn new_document(value: Value) -> Document {
    Document::new_with_data("test_type", value.try_into().unwrap())
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}

pub fn get_values(page: ListPage<Document>) -> Vec<Value> {
    page.items
        .into_iter()
        .map(|item| item.data.into())
        .collect()
}
