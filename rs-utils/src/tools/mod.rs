use std::process::Stdio;

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::process::{Child, Command};
use which::which_all;

use crate::{path_to_string, run_command};

mod backlight;
mod microphone;
mod speakers;
mod touchpad;

pub use self::backlight::Backlight;
pub use self::microphone::Microphone;
pub use self::speakers::Speakers;
pub use self::touchpad::Touchpad;

pub struct NodeJS(String);

impl NodeJS {
    pub fn check() -> Result<Self> {
        find_bin("node")?
            .map(Self)
            .context("NodeJS must be available")
    }
}

pub struct Chromium(String);

impl Chromium {
    pub fn check() -> Result<Self> {
        find_bin("chromium")?
            .map(Self)
            .context("Chromium must be available")
    }

    #[must_use]
    pub fn get_bin_path(&self) -> &str {
        &self.0
    }
}

pub struct Chromedriver {
    bin_path: String,
    pub port: u16,
    pub debug: bool,
}

impl Chromedriver {
    pub fn new() -> Result<Self> {
        let bin_path = find_bin("chromedriver")?.context("Chromedriver must be available")?;

        Ok(Chromedriver {
            bin_path,
            port: 9515,
            debug: false,
        })
    }

    #[must_use]
    pub fn get_bin_path(&self) -> &str {
        &self.bin_path
    }

    pub fn spawn(&self) -> Result<Child> {
        let mut command = Command::new("chromedriver");

        command.arg(format!("--port={}", self.port));

        command.kill_on_drop(true);

        if !self.debug {
            // silence the chromedriver output if not debugging
            command.stdout(Stdio::null());
        }

        command.spawn().context("failed to spawn chromedriver")
    }

    pub fn get_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

pub struct ZStd(String);

impl ZStd {
    pub fn check() -> Result<Self> {
        find_bin("zstd")?
            .map(Self)
            .context("ZStd must be available")
    }

    pub fn compress(&self, src_path: &str, dest_path: &str) -> Result<()> {
        run_command(&self.0, vec!["--compress", src_path, "-o", dest_path])
            .context("failed to run zstd")?;

        Ok(())
    }
}

/// part of ffmpeg
pub struct FFProbe(String);

pub struct FFStats {
    pub duration_ms: u32,
    pub bit_rate: u32,
}

impl FFProbe {
    pub fn check() -> Result<Self> {
        find_bin("ffprobe")?
            .map(Self)
            .context("ffprobe must be available")
    }

    pub fn get_stats(&self, file_path: &str) -> Result<FFStats> {
        let stats = run_command(
            &self.0,
            vec![
                "-loglevel",
                "0",
                "-print_format",
                "json",
                "-show_format",
                file_path,
            ],
        )
        .context("failed to run ffprobe")?;

        let value: Value =
            serde_json::from_str(&stats).context("failed to parse ffprobe output as JSON")?;

        let duration_ms = value["format"]["duration"]
            .as_str()
            .context(".format.duration must be present")?
            .parse::<f32>()
            .context("failed to parse duration")
            .map(|duration| (duration * 1000.0) as u32)?;

        let bit_rate = value["format"]["bit_rate"]
            .as_str()
            .context(".format.bit_rate must be present")?
            .parse()
            .context("failed to parse bit_rate")?;

        Ok(FFStats {
            duration_ms,
            bit_rate,
        })
    }
}

fn find_bin(bin_name: &str) -> Result<Option<String>> {
    let bin_path = which_all(bin_name)
        .context("failed to look for binary")?
        .next()
        .map(path_to_string)
        .transpose()?;

    Ok(bin_path)
}
