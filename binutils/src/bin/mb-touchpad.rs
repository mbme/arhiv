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

use binutils::devices::Touchpad;
use clap::{crate_version, App, Arg};

use rs_utils::log::{error, info, setup_logger};

fn main() {
    setup_logger();

    let app = App::new("mb-touchpad")
        .arg(
            Arg::with_name("notify")
                .short("n")
                .help("Send notification with current touchpad state"),
        )
        .subcommand(App::new("status").about("Print current state of touchpad"))
        .subcommand(App::new("on").about("Enable touchpad"))
        .subcommand(App::new("off").about("Disable touchpad"))
        .subcommand(App::new("toggle").about("Toggle touchpad"))
        .version(crate_version!());

    let matches = app.get_matches();

    let touchpad = Touchpad::find();
    info!("Touchpad id: {}", &touchpad.id);

    let notify = matches.is_present("notify");

    match matches.subcommand_name() {
        Some("status") => {
            touchpad.print_status(notify);
        }
        Some("on") => {
            touchpad.enable(true);
            touchpad.print_status(notify);
        }
        Some("off") => {
            touchpad.disable();
            touchpad.print_status(notify);
        }
        Some("toggle") => {
            touchpad.toggle();
            touchpad.print_status(notify);
        }
        Some(command) => {
            error!("Unexpected command {}", command);
        }
        None => {
            error!("Command is missing");
        }
    }
}
