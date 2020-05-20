use anyhow::*;
use std::process::Command;

pub fn send_notification(message: &str) -> Result<()> {
    let output = Command::new("notify-send")
        .arg("-u")
        .arg("low")
        .arg(message)
        .output()?;

    if !output.status.success() {
        bail!("Command executed with failing error code");
    }

    Ok(())
}
