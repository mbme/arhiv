#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use binutils::devices::Touchpad;
use clap::{crate_version, App, Arg};

fn main() {
    env_logger::init();

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
    log::info!("Touchpad id: {}", &touchpad.id);

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
            log::error!("Unexpected command: {}", command);
        }
        None => {
            log::error!("Command is missing");
        }
    }
}
