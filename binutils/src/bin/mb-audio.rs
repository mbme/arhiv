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

use clap::{crate_version, Arg, Command};

use binutils::devices::{Microphone, Speakers};
use rs_utils::log;

fn main() {
    log::setup_logger();

    let app = Command::new("mb-audio")
        .arg(
            Arg::new("notify")
                .short('n')
                .help("Send notification with current state"),
        )
        .subcommand(
            Command::new("speakers")
                .about("Control speakers")
                .subcommand(Command::new("status").about("Print current volume"))
                .subcommand(Command::new("mute").about("Mute speakers"))
                .subcommand(Command::new("unmute").about("Unmute speakers"))
                .subcommand(Command::new("toggle").about("Toggle mute"))
                .subcommand(Command::new("up").about("Increase volume"))
                .subcommand(Command::new("down").about("Decrease volume")),
        )
        .subcommand(
            Command::new("mic")
                .about("Control microphone")
                .subcommand(Command::new("status").about("Print current state"))
                .subcommand(Command::new("mute").about("Mute microphone"))
                .subcommand(Command::new("unmute").about("Unmute microphone"))
                .subcommand(Command::new("toggle").about("Toggle mute")),
        )
        .version(crate_version!());

    let matches = app.get_matches();
    let notify = matches.is_present("notify");
    let (subcommand, args) = matches.subcommand().expect("subcommand must be provided");

    if subcommand == "speakers" {
        let speakers = Speakers::find();
        match args.subcommand_name() {
            Some("status") => {
                speakers.print_status(notify);
            }
            Some("mute") => {
                speakers.mute();
                speakers.print_status(notify);
            }
            Some("unmute") => {
                speakers.unmute();
                speakers.print_status(notify);
            }
            Some("toggle") => {
                speakers.toggle();
                speakers.print_status(notify);
            }
            Some("up") => {
                speakers.up();
                speakers.print_status(notify);
            }
            Some("down") => {
                speakers.down();
                speakers.print_status(notify);
            }
            Some(command) => {
                log::error!("Unexpected command: {}", command);
            }
            None => {
                log::error!("Command is missing");
            }
        }
        return;
    }

    if subcommand == "mic" {
        let mic = Microphone::find();
        match args.subcommand_name() {
            Some("status") => {
                mic.print_status(notify);
            }
            Some("mute") => {
                mic.mute();
                mic.print_status(notify);
            }
            Some("unmute") => {
                mic.unmute();
                mic.print_status(notify);
            }
            Some("toggle") => {
                mic.toggle();
                mic.print_status(notify);
            }
            Some(command) => {
                log::error!("Unexpected command: {}", command);
            }
            None => {
                log::error!("Command is missing");
            }
        }
        return;
    }

    log::error!("Unknown subcommand {}", subcommand);
}
