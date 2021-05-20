use std::{fs, sync::Arc};

use anyhow::*;

use crate::{entities::Document, Arhiv, Config, ListPage};
use rs_utils::generate_temp_path;

impl Drop for Arhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.config.get_root_dir()).expect("must be able to remove arhiv");
    }
}

fn new_arhiv(config: Config) -> Arc<Arhiv> {
    let arhiv = Arhiv::create(config).expect("must be able to create temp arhiv");

    Arc::new(arhiv)
}

pub fn new_prime() -> Arc<Arhiv> {
    let config = Config::Prime {
        arhiv_id: "test_arhiv".to_string(),
        arhiv_root: generate_temp_path("TempArhiv", ""),
    };

    new_arhiv(config)
}

pub fn new_replica(port: u16) -> Arc<Arhiv> {
    let config = Config::Replica {
        arhiv_id: "test_arhiv".to_string(),
        arhiv_root: generate_temp_path("TempArhiv", ""),
        prime_url: format!("http://localhost:{}", port),
    };

    new_arhiv(config)
}

pub fn empty_document() -> Document {
    Document::new("test_type".to_string(), serde_json::json!({}))
}

pub fn new_document(value: serde_json::Value) -> Document {
    Document::new("test_type".to_string(), value)
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}

pub fn get_values(page: ListPage<Document>) -> Vec<serde_json::Value> {
    page.items.into_iter().map(|item| item.data).collect()
}
