use std::collections::HashMap;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use tokio::signal;

pub use bytes::*;
pub use compression::*;
pub use container::*;
pub use crypto::*;
pub use download::*;
pub use fs::*;
pub use fs_temp::*;
pub use fs_transaction::FsTransaction;
pub use http::*;
pub use iter::*;
pub use json::*;
pub use lock_file::*;
pub use streams::*;
pub use string::*;
pub use time::*;
pub use tools::*;

mod algorithms;
mod bytes;
mod compression;
mod container;
mod crypto;
mod download;
mod fs;
mod fs_temp;
mod fs_transaction;
pub mod full_text_search;
mod http;
pub mod http_server;
pub mod image;
mod iter;
mod json;
pub mod keyring;
mod lock_file;
pub mod log;
pub mod merge;
mod streams;
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

pub fn run_npm<'a>(commands: impl AsRef<[&'a str]>) {
    let commands = commands.as_ref();
    let command_status = Command::new("npm")
        .args(commands)
        .status()
        .expect("failed to run npm command");

    if !command_status.success() {
        println!(
            "cargo:warning=npm {} exit status is {command_status}",
            commands.join(" ")
        );
        process::exit(1);
    }
}

pub fn get_crate_version() -> &'static str {
    option_env!("TYPED_V_VERSION").unwrap_or("dev-build")
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    #[cfg(unix)]
    let interrupt = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("failed to install SIGINT signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let interrupt = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            log::info!("Signals: got Ctrl-C");
        },
        _ = terminate => {
            log::info!("Signals: got SIGTERM");
        },
        _ = interrupt => {
            log::info!("Signals: got SIGINT");
        },
    }
}

pub fn num_cpus() -> Result<usize> {
    let num_cpus = std::thread::available_parallelism()
        .context("Failed to determine the number of available CPUs")?;

    Ok(num_cpus.get())
}

pub fn init_global_rayon_threadpool(num_threads: usize) -> Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .context("Failed to init global rayon thread pool")?;

    Ok(())
}
