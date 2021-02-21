use super::AttachmentData;
use crate::{entities::Id, replica::NetworkService, Arhiv, Config};
use rs_utils::generate_temp_path;

pub struct TestArhiv(Arc<Arhiv>);

impl TestArhiv {
    pub fn new(prime: bool, server_port: u16) -> Self {
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

        TestArhiv(Arc::new(arhiv))
    }

    pub fn unwrap(&self) -> Arc<Arhiv> {
        self.0.clone()
    }

    pub fn get_attachment_data(&self, id: Id) -> AttachmentData {
        let attachment = self.0.get_attachment(&id).unwrap();

        self.0.get_attachment_data(attachment.get_data().hash)
    }

    pub fn get_network_service(&self) -> NetworkService {
        self.0.get_network_service().unwrap()
    }
}

impl Drop for TestArhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.0.config.get_root_dir())
            .expect("must be able to remove arhiv");
    }
}

use std::{ops::Deref, sync::Arc};

impl Deref for TestArhiv {
    type Target = Arhiv;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
