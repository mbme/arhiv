#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

pub use crypto::*;
pub use download::*;
pub use env_capabilities::*;
pub use fs::*;
pub use fs_temp::*;
pub use fs_transaction::FsTransaction;
pub use http::*;
pub use json::*;
pub use markov::Markov;
pub use string::*;

mod crypto;
mod download;
mod env_capabilities;
mod fs;
mod fs_temp;
mod fs_transaction;
mod http;
pub mod http_server;
mod json;
pub mod log;
mod markov;
mod string;

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
