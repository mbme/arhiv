use anyhow::*;
use regex::Regex;
use std::process::Command;

pub fn run_command(command: &str, args: Vec<&str>) -> Result<String> {
    let output = Command::new(command).args(args).output()?;

    if !output.status.success() {
        log::error!(
            "command failed:\n{}\n{}",
            output.status,
            String::from_utf8(output.stderr)?
        );
        bail!("Command executed with failing error code");
    }

    let output_str = String::from_utf8(output.stdout)?;

    Ok(output_str)
}

pub fn send_notification(message: &str) {
    run_command("notify-send", vec!["-u", "low", message])
        .expect("must be able to send notification");
}

pub fn match_str(regex: &Regex, s: &str) -> Option<String> {
    regex.captures(s).map(|captures| {
        captures
            .get(1)
            .expect("group 1 must be present")
            .as_str()
            .to_string()
    })
}
