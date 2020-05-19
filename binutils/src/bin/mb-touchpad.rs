#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::*;
use clap::{crate_version, App};
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

fn main() {
    env_logger::init();

    let app = App::new("mb-touchpad")
        .subcommand(App::new("status").about("Print current state of touchpad"))
        .subcommand(App::new("on").about("Enable touchpad"))
        .subcommand(App::new("off").about("Disable touchpad"))
        .subcommand(App::new("toggle").about("Toggle touchpad"))
        .version(crate_version!());

    let matches = app.get_matches();

    let id = get_touchpad_id().expect("must be able to find touchpad");
    log::info!("Touchpad id: {}", &id);

    match matches.subcommand_name() {
        Some("status") => {
            let enabled = is_device_enabled(&id).expect("must be able to read touchpad status");
            println!(
                "Touchpad is {}",
                if enabled { "enabled" } else { "disabled " }
            );
        }
        Some("on") => {
            enable_device(&id, true).expect("must be able to enable touchpad");
        }
        Some("off") => {
            enable_device(&id, false).expect("must be able to disable touchpad");
        }
        Some("toggle") => {
            let enabled = is_device_enabled(&id).expect("must be able to read touchpad status");
            enable_device(&id, !enabled).expect("must be able to toggle touchpad");
        }
        Some(command) => {
            log::error!("Unexpected command: {}", command);
        }
        None => {
            log::error!("Command is missing");
        }
    }
}
