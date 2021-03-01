use crate::{entities::Document, Arhiv, Config, ListPage};
use anyhow::*;
use rs_utils::generate_temp_path;
use std::{fs, sync::Arc};

impl Drop for Arhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.config.get_root_dir()).expect("must be able to remove arhiv");
    }
}

fn new_arhiv(prime: bool, server_port: u16) -> Arc<Arhiv> {
    let config = {
        if prime {
            Config::Prime {
                arhiv_id: "test_arhiv".to_string(),
                arhiv_root: generate_temp_path("TempArhiv", ""),
                server_port,
            }
        } else {
            Config::Replica {
                arhiv_id: "test_arhiv".to_string(),
                arhiv_root: generate_temp_path("TempArhiv", ""),
                prime_url: format!("http://localhost:{}", server_port),
            }
        }
    };

    let arhiv = Arhiv::create(config).expect("must be able to create temp arhiv");

    Arc::new(arhiv)
}

pub fn new_prime() -> Arc<Arhiv> {
    new_arhiv(true, 0)
}

pub fn new_replica(port: u16) -> Arc<Arhiv> {
    new_arhiv(false, port)
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
