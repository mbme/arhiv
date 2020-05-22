#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use binutils::devices::{Microphone, Speakers};
use clap::{crate_version, App, Arg};

fn main() {
    env_logger::init();

    let app = App::new("mb-audio")
        .arg(
            Arg::with_name("notify")
                .short("n")
                .help("Send notification with current state"),
        )
        .subcommand(
            App::new("speakers")
                .about("Control speakers")
                .subcommand(App::new("status").about("Print current volume"))
                .subcommand(App::new("mute").about("Mute speakers"))
                .subcommand(App::new("unmute").about("Unmute speakers"))
                .subcommand(App::new("toggle").about("Toggle mute"))
                .subcommand(App::new("up").about("Increase volume"))
                .subcommand(App::new("down").about("Decrease volume")),
        )
        .subcommand(
            App::new("mic")
                .about("Control microphone")
                .subcommand(App::new("status").about("Print current state"))
                .subcommand(App::new("mute").about("Mute microphone"))
                .subcommand(App::new("unmute").about("Unmute microphone"))
                .subcommand(App::new("toggle").about("Toggle mute")),
        )
        .version(crate_version!());

    let matches = app.get_matches();
    let notify = matches.is_present("notify");
    let (subcommand, args) = matches.subcommand();

    if subcommand == "speakers" {
        let speakers = Speakers::find();
        match args.and_then(|args| args.subcommand_name()) {
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
        match args.and_then(|args| args.subcommand_name()) {
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
