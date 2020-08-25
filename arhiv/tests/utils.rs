use anyhow::*;
use arhiv::{Arhiv, Config};
use std::env;
use std::fs;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SERVERS_COUNTER: AtomicU16 = AtomicU16::new(0);

fn generate_port() -> u16 {
    9876 + SERVERS_COUNTER.fetch_add(1, Ordering::Relaxed)
}

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
    let primary_url = {
        if prime {
            None
        } else {
            Some(format!("http://localhost:{}", server_port))
        }
    };

    let config = Config {
        is_prime: prime,
        arhiv_root: generate_temp_dir("TempArhiv"),
        primary_url,
        server_port,
    };

    Arhiv::create(config).expect("must be able to create temp arhiv")
}

pub fn new_prime() -> Arhiv {
    new_arhiv(true, generate_port())
}

pub fn new_replica() -> Arhiv {
    new_arhiv(false, generate_port())
}

pub fn new_arhiv_pair() -> (Arhiv, Arhiv) {
    let port = generate_port();

    (new_arhiv(true, port), new_arhiv(false, port))
}

pub fn are_equal_files(src: &str, dst: &str) -> Result<bool> {
    Ok(fs::read(src)? == fs::read(dst)?)
}
