#![deny(clippy::all)]

use clap::{Parser, Subcommand};

use rs_utils::{get_crate_version, log, Backlight};

/// Control brightness of the screen of the laptop
#[derive(Parser, Debug)]
#[clap(version = get_crate_version(), about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// Show notification with the current brightness level
    #[clap(short, action, global = true)]
    notify: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print current brightness level
    Status,
    /// Increase backlight brightness
    #[clap(name = "inc")]
    Increase,
    /// Decrease backlight brightness
    #[clap(name = "dec")]
    Decrease,
}

fn main() {
    log::setup_logger();

    let args = Args::parse();

    match args.command {
        Command::Increase => {
            Backlight::inc();
            Backlight::print_status(args.notify);
        }
        Command::Decrease => {
            Backlight::dec();
            Backlight::print_status(args.notify);
        }
        Command::Status => {
            Backlight::print_status(args.notify);
        }
    }
}
