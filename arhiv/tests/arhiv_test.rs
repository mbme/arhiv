use arhiv::utils::generate_temp_dir;
use arhiv::{Arhiv, Config};
use std::fs;

struct TempArhiv {
    pub arhiv: arhiv::Arhiv,
}

impl TempArhiv {
    pub fn new(prime: bool) -> TempArhiv {
        let config = Config {
            prime,
            arhiv_root: generate_temp_dir("TempArhiv"),
            primary_url: None,
            server_port: 0,
        };

        let arhiv = Arhiv::create(config).expect("must be able to create temp arhiv");

        TempArhiv { arhiv }
    }
}

impl Drop for TempArhiv {
    fn drop(&mut self) {
        fs::remove_dir_all(self.arhiv.get_root_dir()).expect("must be able to remove arhiv");
    }
}

fn create_temp_dir() {}

#[test]
fn it_works() {}
