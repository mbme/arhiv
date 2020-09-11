use anyhow::*;
use arhiv::{entities::Document, Arhiv, Config};
use rs_utils::generate_temp_path;
use std::fs;

fn new_arhiv(prime: bool, server_port: u16) -> Arhiv {
    let prime_url = {
        if prime {
            None
        } else {
            Some(format!("http://localhost:{}", server_port))
        }
    };

    let config = Config {
        arhiv_root: generate_temp_path("TempArhiv", ""),
        prime_url,
        server_port,
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

pub fn new_document() -> Document {
    Document::new("test")
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}
