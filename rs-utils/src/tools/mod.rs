use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::process::{Child, Command};
use tokio::time::{sleep, Instant};
use which::which_all;

use crate::{path_to_string, run_command};

pub struct NodeJS(String);

impl NodeJS {
    pub fn check() -> Result<Self> {
        find_bin("node")?
            .map(Self)
            .context("NodeJS must be available")
    }

    #[must_use]
    pub fn get_bin_path(&self) -> &str {
        &self.0
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

    pub async fn wait_for_ready(&self, timeout_secs: u64) -> Result<bool> {
        let start_time = Instant::now();

        while Instant::now().duration_since(start_time).as_secs() < timeout_secs {
            match self.is_ready().await {
                Ok(is_ready) => {
                    if is_ready {
                        return Ok(true);
                    }
                }
                Err(err) => {
                    if self.debug {
                        println!("Chromedriver not ready yet: {err}")
                    }
                }
            };

            sleep(Duration::from_millis(500)).await;
        }

        Err(anyhow!(
            "chromedriver failed to start in {timeout_secs} seconds"
        ))
    }

    async fn is_ready(&self) -> Result<bool> {
        let url = self.get_url();

        // https://www.w3.org/TR/webdriver2/#dfn-status
        let status_url = format!("{url}/status");

        let response = Client::new().get(&status_url).send().await?;
        let response = response.error_for_status()?;

        let body = response.text().await?;
        if self.debug {
            println!("status body: {body}");
        }

        let mut body: Value = serde_json::from_str(&body).context("failed to parse status body")?;

        #[derive(Deserialize)]
        struct ChromedriverStatus {
            ready: bool,
            message: String,
        }

        let status: ChromedriverStatus =
            serde_json::from_value(body["value"].take()).context("failed to parse status body")?;

        if status.ready {
            Ok(true)
        } else {
            Err(anyhow!("chromedriver not ready yet: {}", status.message))
        }
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
