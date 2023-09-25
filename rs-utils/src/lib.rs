#![deny(clippy::all)]

use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::process;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use anyhow::{anyhow, bail, Context, Result};
use gethostname::gethostname;
use image::GenericImageView;
use regex::Regex;

pub use crypto::*;
pub use download::*;
pub use fs::*;
pub use fs_temp::*;
pub use fs_transaction::FsTransaction;
pub use http::*;
pub use json::*;
pub use string::*;
pub use time::*;
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
pub mod mdns;
mod string;
mod time;
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
        println!("cargo:warning=yarn {command} exit status is {command_status}");
        process::exit(1);
    }
}

/// returns (width, height)
pub fn get_image_size(file_path: &str) -> Result<(u32, u32)> {
    let img = image::io::Reader::open(file_path)
        .context("failed to open image")?
        .with_guessed_format()
        .context("failed to guess format")?
        .decode()
        .context("failed to decode image")?;

    Ok(img.dimensions())
}

// returns png image
pub fn scale_image(file_path: &str, max_w: Option<u32>, max_h: Option<u32>) -> Result<Vec<u8>> {
    const MAX_THUMBNAIL_SIZE: u32 = 96;

    let start_time = SystemTime::now();

    let img_size = get_file_size(file_path)?;
    let img = image::io::Reader::open(file_path)
        .context("failed to open image")?
        .with_guessed_format()
        .context("failed to guess format")?
        .decode()
        .context("failed to decode image")?;

    let mut bytes: Vec<u8> = Vec::new();

    let (width, height) = img.dimensions();

    let max_w = max_w.unwrap_or(width);
    let max_h = max_h.unwrap_or(height);

    let resized_img = if max_w <= MAX_THUMBNAIL_SIZE || max_h <= MAX_THUMBNAIL_SIZE {
        img.thumbnail(max_w, max_h)
    } else {
        img.resize(max_w, max_h, image::imageops::FilterType::Lanczos3)
    };

    resized_img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;

    let resized_img_size = resized_img.as_bytes().len();

    let end_time = SystemTime::now();
    let duration = end_time.duration_since(start_time)?;

    log::debug!(
        "scaled image from {img_size} bytes to {resized_img_size} bytes: to {}% in {} seconds",
        resized_img_size as u64 * 100 / img_size,
        duration.as_secs(),
    );

    Ok(bytes)
}

pub fn send_notification(message: &str) {
    run_command("notify-send", vec!["-u", "low", message])
        .expect("must be able to send notification");
}

#[must_use]
pub fn match_str(regex: &Regex, s: &str) -> Option<String> {
    regex.captures(s).map(|captures| {
        captures
            .get(1)
            .expect("group 1 must be present")
            .as_str()
            .to_string()
    })
}

pub fn get_crate_version() -> &'static str {
    option_env!("TYPED_V_VERSION").unwrap_or("dev-build")
}

pub fn get_hostname() -> Result<String> {
    gethostname().into_string().map_err(|err| {
        anyhow!(
            "failed to convert hostname into string: {}",
            err.to_string_lossy()
        )
    })
}
