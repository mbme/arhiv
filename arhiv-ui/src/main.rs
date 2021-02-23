use std::{env, process::Command};

use clap::{crate_version, App, Arg};
use rs_utils::log::setup_logger;
use server::start_server;

mod commander;
mod server;

#[tokio::main]
async fn main() {
    setup_logger();

    let app = App::new("arhiv-ui")
        .arg(
            Arg::with_name("open")
                .long("open")
                .takes_value(true)
                .help("Open app using provided browser or $BROWSER env variable"),
        )
        .version(crate_version!());

    let matches = app.get_matches();

    let browser = {
        if let Some(browser) = matches.value_of("open") {
            browser.to_string()
        } else {
            env::var("BROWSER").expect("failed to read BROWSER env variable")
        }
    };

    let (join_handle, addr) = start_server().await;

    Command::new(&browser)
        .arg(format!("http://{}", addr))
        .spawn()
        .expect(&format!("failed to run browser {}", browser));

    join_handle.await.expect("must join");
}
