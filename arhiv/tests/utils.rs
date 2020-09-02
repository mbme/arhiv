use anyhow::*;
use arhiv::{entities::Document, Arhiv, Config};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_temp_dir(prefix: &str) -> String {
    let mut path = env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    path.push(format!("{}-{}", prefix, nanos));

    path.to_str()
        .expect("must be able to convert path to string")
        .to_string()
}

fn new_arhiv(prime: bool, server_port: u16) -> Arhiv {
    let prime_url = {
        if prime {
            None
        } else {
            Some(format!("http://localhost:{}", server_port))
        }
    };

    let config = Config {
        arhiv_root: generate_temp_dir("TempArhiv"),
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
