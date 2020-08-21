use arhiv::{Arhiv, Config};
use std::env;
use std::fs;
use std::ops::Deref;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TempArhiv {
    arhiv: Arhiv,
}

impl TempArhiv {
    pub fn new(prime: bool) -> TempArhiv {
        let config = Config {
            is_prime: prime,
            arhiv_root: generate_temp_dir("TempArhiv"),
            primary_url: None,
            server_port: 0,
        };

        let arhiv = Arhiv::create(config).expect("must be able to create temp arhiv");

        TempArhiv { arhiv }
    }
}

impl Drop for TempArhiv {
    // teardown
    fn drop(&mut self) {
        fs::remove_dir_all(self.arhiv.get_root_dir()).expect("must be able to remove arhiv");
    }
}

impl Deref for TempArhiv {
    type Target = Arhiv;

    fn deref(&self) -> &Arhiv {
        &self.arhiv
    }
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
