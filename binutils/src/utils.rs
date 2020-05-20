use anyhow::*;
use std::process::Command;

pub fn run_command(command: &str, args: Vec<&str>) -> Result<String> {
    let output = Command::new(command).args(args).output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    let output_str = String::from_utf8(output.stdout)?;

    Ok(output_str)
}

pub fn send_notification(message: &str) -> Result<()> {
    run_command("notify-send", vec!["-u", "low", message])?;

    Ok(())
}
