use std::{convert::TryInto, fs, sync::Arc};

use anyhow::*;

use crate::{
    entities::Document,
    schema::{DataDescription, Field, FieldType},
    Arhiv, Config, ListPage,
};
use rs_utils::generate_temp_path;

impl Drop for Arhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.config.arhiv_root).expect("must be able to remove arhiv");
    }
}

fn new_arhiv(config: Config, prime: bool) -> Arc<Arhiv> {
    let mut arhiv = Arhiv::create(config, "test-arhiv".to_string(), prime)
        .expect("must be able to create temp arhiv");

    arhiv.schema.modules.push(DataDescription {
        document_type: "test_type",
        is_internal: false,
        collection_of: None,
        fields: vec![Field {
            name: "ref",
            field_type: FieldType::Ref("attachment"),
            optional: true,
        }],
    });

    Arc::new(arhiv)
}

pub fn new_prime() -> Arc<Arhiv> {
    let config = Config {
        arhiv_root: generate_temp_path("TempArhiv", ""),
        ..Config::default()
    };

    new_arhiv(config, true)
}

pub fn new_replica(port: u16) -> Arc<Arhiv> {
    let config = Config {
        arhiv_root: generate_temp_path("TempArhiv", ""),
        prime_url: format!("http://localhost:{}", port),
        ..Config::default()
    };

    new_arhiv(config, false)
}

pub fn empty_document() -> Document {
    Document::new("test_type")
}

pub fn new_document(value: serde_json::Value) -> Document {
    Document::new_with_data("test_type", value.try_into().unwrap())
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}

pub fn get_values(page: ListPage<Document>) -> Vec<serde_json::Value> {
    page.items
        .into_iter()
        .map(|item| item.data.into())
        .collect()
}
