use anyhow::*;
pub use config::*;
pub use crypto::*;
pub use fs::*;
pub use fs_transaction::FsTransaction;
use std::env;
use std::process::Command;
pub use string::*;

mod config;
mod crypto;
mod fs;
mod fs_transaction;
mod string;

pub fn project_relpath(subpath: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), subpath)
}

// development or production
const MODE: Option<&'static str> = option_env!("MODE");

pub fn is_production_mode() -> bool {
    MODE.unwrap_or("development") == "production"
}

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
