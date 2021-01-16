use anyhow::*;
use arhiv::{entities::Document, Arhiv, Config, ListPage};
use rs_utils::generate_temp_path;
use std::fs;

fn new_arhiv(prime: bool, server_port: u16) -> Arhiv {
    let config = {
        if prime {
            Config::Prime {
                arhiv_root: generate_temp_path("TempArhiv", ""),
                server_port,
            }
        } else {
            Config::Replica {
                arhiv_root: generate_temp_path("TempArhiv", ""),
                prime_url: format!("http://localhost:{}", server_port),
            }
        }
    };

    Arhiv::create(prime, config).expect("must be able to create temp arhiv")
}

pub fn new_prime() -> Arhiv {
    new_arhiv(true, 0)
}

pub fn new_replica() -> Arhiv {
    new_arhiv(false, 0)
}

pub fn new_replica_with_port(port: u16) -> Arhiv {
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
