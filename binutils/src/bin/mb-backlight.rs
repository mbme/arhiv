#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use clap::{crate_version, App, Arg};

use binutils::devices::Backlight;
use rs_utils::log::setup_logger;

fn main() {
    setup_logger();

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
        Some("inc") => {
            Backlight::inc();
            Backlight::print_status(notify);
        }
        Some("dec") => {
            Backlight::dec();
            Backlight::print_status(notify);
        }
        _ => {
            Backlight::print_status(notify);
        }
    }
}
