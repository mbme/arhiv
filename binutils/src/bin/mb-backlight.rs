#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use binutils::devices::Backlight;
use clap::{crate_version, App, Arg};

fn main() {
    env_logger::init();

    let app = App::new("mb-backlight")
        .arg(
            Arg::with_name("notify")
                .short("n")
                .help("Send notification with current backlight state"),
        )
        .subcommand(App::new("status").about("Print current state of backlight"))
        .subcommand(App::new("inc").about("Increase backlight"))
        .subcommand(App::new("dec").about("Decrease backlight"))
        .version(crate_version!());

    let matches = app.get_matches();

    let notify = matches.is_present("notify");

    match matches.subcommand_name() {
        Some("status") => {
            Backlight::print_status(notify);
        }
        Some("inc") => {
            Backlight::inc();
            Backlight::print_status(notify);
        }
        Some("dec") => {
            Backlight::dec();
            Backlight::print_status(notify);
        }
        Some(command) => {
            log::error!("Unexpected command: {}", command);
        }
        None => {
            log::error!("Command is missing");
        }
    }
}
