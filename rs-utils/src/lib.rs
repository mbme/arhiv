use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::*;

pub use crypto::*;
pub use fs::*;
pub use fs_temp::*;
pub use fs_transaction::FsTransaction;
pub use http::*;
pub use json::*;
pub use markov::Markov;
pub use string::*;

mod crypto;
mod fs;
mod fs_temp;
mod fs_transaction;
mod http;
mod json;
pub mod log;
mod markov;
mod string;

pub fn project_relpath(subpath: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), subpath)
}

pub fn run_command(command: &str, args: Vec<&str>) -> Result<String> {
    let output = Command::new(command).args(args).output()?;

    if !output.status.success() {
        let err_str = String::from_utf8(output.stderr)?;
        log::error!("command failed:\n{}\n{}", output.status, err_str);
        bail!("Command executed with failing error code");
    }

    let output_str = String::from_utf8(output.stdout)?;

    Ok(output_str)
}

pub fn run_js_script(script: impl AsRef<str>, args: Vec<&str>) -> Result<String> {
    let script = script.as_ref();

    let mut child = Command::new("node")
        .arg("-") // read script from stdin
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut child_stdin) = child.stdin.take() {
        child_stdin.write_all(script.as_bytes())?;
    } else {
        bail!("failed to run js script: can't write to stdin")
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        let output_str = String::from_utf8(output.stdout)?;

        Ok(output_str)
    } else {
        let err_str = String::from_utf8(output.stderr)?;

        log::error!(
            "failed to run js script: exit code {}\n{}",
            output.status,
            err_str
        );

        bail!("failed to run js script: exit code {}", output.status)
    }
}

pub fn is_image_filename(filename: impl AsRef<str>) -> bool {
    let filename = filename.as_ref();

    filename.ends_with(".png")
        || filename.ends_with(".jpg")
        || filename.ends_with(".jpeg")
        || filename.ends_with(".svg")
}
