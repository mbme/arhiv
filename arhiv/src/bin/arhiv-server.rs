#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::sync::Arc;

use tracing_subscriber::{fmt, layer::SubscriberExt};

use arhiv::{start_server, Arhiv};

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

    let (file_writer, _guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(
        arhiv.config.get_root_dir(),
        "arhiv-server.log",
    ));
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter("arhiv=debug,hyper=info")
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer)),
    )
    .expect("Unable to set global tracing subscriber");

    let (join_handle, _, _) = start_server(arhiv);

    join_handle.await.expect("must join");
}
