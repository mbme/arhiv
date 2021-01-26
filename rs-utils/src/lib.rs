use anyhow::*;
pub use config::*;
pub use crypto::*;
pub use fs::*;
pub use fs_temp::*;
pub use fs_transaction::FsTransaction;
pub use markov::Markov;
use std::env;
use std::process::Command;
pub use string::*;
use tracing::{error, Level};

mod config;
mod crypto;
mod fs;
mod fs_temp;
mod fs_transaction;
mod markov;
mod string;

pub fn project_relpath(subpath: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), subpath)
}

pub fn run_command(command: &str, args: Vec<&str>) -> Result<String> {
    let output = Command::new(command).args(args).output()?;

    if !output.status.success() {
        let err_str = String::from_utf8(output.stderr)?;
        error!("command failed:\n{}\n{}", output.status, err_str);
        bail!("Command executed with failing error code");
    }

    let output_str = String::from_utf8(output.stdout)?;

    Ok(output_str)
}

pub fn setup_logger() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

pub fn get_log_level(log_level: u64) -> Level {
    match log_level {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    }
}
