#![deny(clippy::all)]

use std::collections::HashMap;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

pub use crypto::*;
pub use download::*;
pub use fs::*;
pub use fs_temp::*;
pub use fs_transaction::FsTransaction;
pub use http::*;
pub use json::*;
pub use markov::Markov;
pub use string::*;
pub use tools::*;

mod crypto;
mod download;
mod fs;
mod fs_temp;
mod fs_transaction;
mod http;
pub mod http_server;
mod json;
pub mod log;
mod markov;
mod string;
mod tools;

pub fn run_command(command: &str, args: Vec<&str>) -> Result<String> {
    run_command_with_envs(command, args, HashMap::new())
}

pub fn run_command_with_envs(
    command: &str,
    args: Vec<&str>,
    envs: HashMap<&str, &str>,
) -> Result<String> {
    let output = Command::new(command)
        .args(args)
        .envs(envs)
        .output()
        .context("failed to execute command")?;

    if !output.status.success() {
        let err_str = String::from_utf8(output.stderr)?;
        log::error!("command failed:\n{}\n{}", output.status, err_str);
        bail!("Command '{}' executed with failing error code", command);
    }

    let output_str =
        String::from_utf8(output.stdout).context("failed to convert stdout to string")?;

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

pub fn run_yarn(command: &str) {
    let command_status = Command::new("yarn")
        .arg(command)
        .status()
        .expect("failed to run yarn command");

    if !command_status.success() {
        println!(
            "cargo:warning=yarn {} exit status is {}",
            command, command_status
        );
        process::exit(1);
    }
}

/// returns (width, height)
pub fn get_image_size(file_path: &str) -> Result<(usize, usize)> {
    let dimensions = imagesize::size(file_path).context("Failed to determine image size")?;

    Ok((dimensions.width, dimensions.height))
}
