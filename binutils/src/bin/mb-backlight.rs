#![deny(clippy::all)]

use clap::{Arg, Command};

use rs_utils::{get_crate_version, log, Backlight};

fn main() {
    log::setup_logger();

    let app = Command::new("mb-backlight")
        .arg(
            Arg::new("notify")
                .short('n')
                .help("Send notification with current backlight state"),
        )
        .subcommand(Command::new("status").about("Print current state of backlight"))
        .subcommand(Command::new("inc").about("Increase backlight"))
        .subcommand(Command::new("dec").about("Decrease backlight"))
        .version(get_crate_version());

    let matches = app.get_matches();

    let notify = matches.contains_id("notify");

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
