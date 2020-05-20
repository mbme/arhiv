#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::*;
use binutils::notify::send_notification;
use clap::{crate_version, App, Arg};
use lazy_static::*;
use regex::Regex;
use std::process::Command;

fn get_touchpad_id() -> Result<String> {
    let output = Command::new("xinput").arg("list").arg("--short").output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    lazy_static! {
        static ref ID_RE: Regex = Regex::new(r"(?i)touchpad.*\sid=([0-9]+)").unwrap();
    }

    let mut ids: Vec<String> = vec![];

    String::from_utf8(output.stdout)?.lines().for_each(|line| {
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

fn is_device_enabled(device_id: &str) -> Result<bool> {
    let output = Command::new("xinput")
        .arg("list-props")
        .arg(device_id)
        .output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    let enabled = String::from_utf8(output.stdout)?
        .lines()
        .find(|line| line.find("Device Enabled").is_some() && line.ends_with("1"))
        .is_some();

    Ok(enabled)
}

fn enable_device(device_id: &str, enable: bool) -> Result<()> {
    let output = Command::new("xinput")
        .arg(if enable { "enable" } else { "disable" })
        .arg(device_id)
        .output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    Ok(())
}

struct Touchpad {
    id: String,
}

impl Touchpad {
    fn find() -> Self {
        let id = get_touchpad_id().expect("must be able to find touchpad");

        Touchpad { id }
    }

    fn is_enabled(&self) -> bool {
        is_device_enabled(&self.id).expect("must be able to read touchpad status")
    }

    fn enable(&self) {
        enable_device(&self.id, true).expect("must be able to enable touchpad");
    }

    fn disable(&self) {
        enable_device(&self.id, false).expect("must be able to disable touchpad");
    }

    fn toggle(&self) {
        enable_device(&self.id, !self.is_enabled()).expect("must be able to toggle touchpad");
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
            touchpad.enable();
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
