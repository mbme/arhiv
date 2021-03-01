#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env,
    process::{Command, Stdio},
};

use arhiv::start_ui_server;
use clap::{crate_version, App, Arg};
use rs_utils::log::setup_logger_with_level;

#[tokio::main]
async fn main() {
    let app = App::new("arhiv-ui")
        .arg(
            Arg::with_name("open")
                .long("open")
                .takes_value(true)
                .help("Open app using provided browser or $BROWSER env variable"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Increases logging verbosity each use for up to 2 times"),
        )
        .version(crate_version!());

    let matches = app.get_matches();

    setup_logger_with_level(matches.occurrences_of("verbose"));

    let browser = {
        if let Some(browser) = matches.value_of("open") {
            browser.to_string()
        } else {
            env::var("BROWSER").expect("failed to read BROWSER env variable")
        }
    };

    let (join_handle, addr) = start_ui_server().await;

    Command::new(&browser)
        .arg(format!("http://{}", addr))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect(&format!("failed to run browser {}", browser));

    join_handle.await.expect("must join");
}
