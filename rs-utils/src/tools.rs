use anyhow::{Context, Result};
use serde_json::Value;
use which::which_all;

use crate::{path_to_string, run_command};

pub struct NodeJS(String);

impl NodeJS {
    pub fn check() -> Result<Self> {
        find_bin("nodejs")?
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

impl FFProbe {
    pub fn check() -> Result<Self> {
        find_bin("ffprobe")?
            .map(Self)
            .context("ffprobe must be available")
    }

    fn get_stats(&self, file_path: &str) -> Result<Value> {
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

        serde_json::from_str(&stats).context("failed to parse ffprobe output as JSON")
    }

    pub fn get_duration(&self, file_path: &str) -> Result<f32> {
        let stats = self.get_stats(file_path)?;

        stats["format"]["duration"]
            .as_str()
            .context(".format.duration must be present")?
            .parse()
            .context("failed to parse duration")
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
