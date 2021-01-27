#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{path::Path, sync::Arc};

use arhiv::{start_server, Arhiv};
use rs_utils::log::setup_server_logger;

#[tokio::main]
async fn main() {
    if cfg!(not(feature = "production-mode")) {
        println!("DEBUG MODE");
    }

    let arhiv = Arc::new(Arhiv::must_open());
    if !arhiv
        .get_status()
        .expect("must be able to get status")
        .db_status
        .is_prime
    {
        panic!("server must be started on prime instance");
    }

    let log_file = Path::new(arhiv.config.get_root_dir()).join("arhiv-server.log");
    setup_server_logger(log_file);

    let (join_handle, _, _) = start_server(arhiv);

    join_handle.await.expect("must join");
}
