use crate::utils::{match_str, send_notification};
use anyhow::*;
use lazy_static::*;
use regex::Regex;
use rs_utils::run_command;

pub struct Touchpad {
    pub id: String,
}

impl Touchpad {
    pub fn find() -> Self {
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
            let result = match_str(&ID_RE, line);

            if let Some(id) = result {
                log::info!("{}", &line);

                ids.push(id);
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

    pub fn is_enabled(&self) -> bool {
        let output = run_command("xinput", vec!["list-props", &self.id])
            .expect("must be able to read touchpad status");

        output
            .lines()
            .find(|line| line.find("Device Enabled").is_some() && line.ends_with("1"))
            .is_some()
    }

    pub fn enable(&self, enable: bool) {
        let arg = if enable { "enable" } else { "disable" };

        run_command("xinput", vec![arg, &self.id])
            .expect(&format!("must be able to {} touchpad", arg));
    }

    pub fn disable(&self) {
        self.enable(false)
    }

    pub fn toggle(&self) {
        self.enable(!self.is_enabled());
    }

    pub fn print_status(&self, notify: bool) {
        let message = format!(
            "Touchpad is {}",
            if self.is_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        );

        println!("{}", &message);

        if notify {
            send_notification(&message);
        }
    }
}
