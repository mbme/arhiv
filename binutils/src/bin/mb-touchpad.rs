#![deny(clippy::all)]

use clap::{Arg, Command};

use rs_utils::{get_crate_version, log, Touchpad};

fn main() {
    log::setup_logger();

    let app = Command::new("mb-touchpad")
        .arg(
            Arg::new("notify")
                .short('n')
                .help("Send notification with current touchpad state"),
        )
        .subcommand(Command::new("status").about("Print current state of touchpad"))
        .subcommand(Command::new("on").about("Enable touchpad"))
        .subcommand(Command::new("off").about("Disable touchpad"))
        .subcommand(Command::new("toggle").about("Toggle touchpad"))
        .version(get_crate_version());

    let matches = app.get_matches();

    let touchpad = Touchpad::find();
    log::info!("Touchpad id: {}", &touchpad.id);

    let notify = matches.contains_id("notify");

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
            log::error!("Unexpected command {}", command);
        }
        None => {
            log::error!("Command is missing");
        }
    }
}
