use std::{env, process};

use anyhow::{Context, Result};

use arhiv::{ArhivOptions, ArhivServer};
use baza::DEV_MODE;
use baza_common::{log, shutdown_signal};

pub(crate) async fn handle_server_command(port: u16, json: bool, browser: bool) -> Result<()> {
    let server = ArhivServer::start(ArhivOptions::new_desktop(), port).await?;
    let server_info = server.get_info();

    if json {
        eprintln!(
            "@@SERVER_INFO: {}",
            serde_json::to_string(server_info).expect("Failed to serialize ServerInfo")
        );
    }

    if browser {
        let browser = env::var("BROWSER").context("Failed to read $BROWSER env variable")?;

        log::info!("Browser URL: {}", server_info.browser_url);
        launch_browser(&browser, &server_info.browser_url)?;
    } else if DEV_MODE {
        log::info!("Dev server url: {}", server_info.browser_url);
    }

    shutdown_signal().await;

    server.shutdown().await?;

    Ok(())
}

fn launch_browser(browser: &str, browser_url: &str) -> Result<()> {
    let mut command = process::Command::new(browser);
    command
        .arg(browser_url)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        command.process_group(0);
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to run browser {browser}"))?;

    let _ = std::thread::spawn(move || {
        if let Err(err) = child.wait() {
            log::warn!("Failed to wait for browser process: {err}");
        }
    });

    Ok(())
}
