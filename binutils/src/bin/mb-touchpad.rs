#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::*;
use binutils::utils::{run_command, send_notification};
use clap::{crate_version, App, Arg};
use lazy_static::*;
use regex::Regex;

struct Touchpad {
    id: String,
}

impl Touchpad {
    fn find() -> Self {
        let id = Touchpad::get_touchpad_id().expect("must be able to find touchpad");

        Touchpad { id }
    }

    fn get_touchpad_id() -> Result<String> {
        let output = run_command("xinput", vec!["list", "--short"])?;

        lazy_static! {
            static ref ID_RE: Regex = Regex::new(r"(?i)touchpad.*\sid=([0-9]+)").unwrap();
        }

        let mut ids: Vec<String> = vec![];

        output.lines().for_each(|line| {
            let result = ID_RE.captures(line);

            if let Some(captures) = result {
                log::info!("{}", &line);

                ids.push(captures.get(1).unwrap().as_str().to_string());
            }
        });

        if ids.len() > 1 {
            bail!("Found multiple touchpads");
        }

        if ids.is_empty() {
            bail!("Touchpad not found");
        }

        Ok(ids.remove(0))
    }

    fn is_enabled(&self) -> bool {
        let output = run_command("xinput", vec!["list-props", &self.id])
            .expect("must be able to read touchpad status");

        output
            .lines()
            .find(|line| line.find("Device Enabled").is_some() && line.ends_with("1"))
            .is_some()
    }

    fn enable(&self, enable: bool) {
        let arg = if enable { "enable" } else { "disable" };

        run_command("xinput", vec![arg, &self.id])
            .expect(&format!("must be able to {} touchpad", arg));
    }

    fn disable(&self) {
        self.enable(false)
    }

    fn toggle(&self) {
        self.enable(!self.is_enabled());
    }
}

fn print_status(touchpad: &Touchpad, notify: bool) {
    let message = format!(
        "Touchpad is {}",
        if touchpad.is_enabled() {
            "enabled"
        } else {
            "disabled"
        }
    );

    println!("{}", &message);

    if notify {
        send_notification(&message).expect("must be able to send notification");
    }
}

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
            print_status(&touchpad, notify);
        }
        Some("on") => {
            touchpad.enable(true);
            print_status(&touchpad, notify);
        }
        Some("off") => {
            touchpad.disable();
            print_status(&touchpad, notify);
        }
        Some("toggle") => {
            touchpad.toggle();
            print_status(&touchpad, notify);
        }
        Some(command) => {
            log::error!("Unexpected command: {}", command);
        }
        None => {
            log::error!("Command is missing");
        }
    }
}
