#![deny(clippy::all)]

use clap::{Parser, Subcommand};

use rs_utils::{get_crate_version, log, Touchpad};

#[derive(Parser, Debug)]
#[clap(version = get_crate_version(), about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// Show notification with the current touchpad state
    #[clap(short, action, global = true)]
    notify: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print current touchpad state
    Status,
    /// Enable touchpad
    On,
    /// Disable touchpad
    Off,
    /// Toggle touchpad
    Toggle,
}

fn main() {
    log::setup_logger();

    let args = Args::parse();

    let touchpad = Touchpad::find();
    log::info!("Touchpad id: {}", &touchpad.id);

    match args.command {
        Command::Status => {
            touchpad.print_status(args.notify);
        }
        Command::On => {
            touchpad.enable(true);
            touchpad.print_status(args.notify);
        }
        Command::Off => {
            touchpad.disable();
            touchpad.print_status(args.notify);
        }
        Command::Toggle => {
            touchpad.toggle();
            touchpad.print_status(args.notify);
        }
    }
}
