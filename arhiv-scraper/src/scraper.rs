use std::process::Stdio;

use anyhow::{ensure, Context, Error, Result};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, Command},
    try_join,
};

use arhiv_core::{
    definitions::Attachment,
    entities::{Document, DocumentData},
    Arhiv, ArhivTransaction,
};
use rs_utils::{log, Download, EnvCapabilities, TempFile};

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ScraperAction {
    CreateAttachment {
        url: String,
    },
    CreateDocument {
        document_type: String,
        data: DocumentData,
    },
}

fn get_script_temp_file() -> Result<TempFile> {
    let script = include_str!("../dist/bundle.js");

    // TODO use "shared memory file" shm_open
    let temp_file = TempFile::new_with_details("arhiv-scrape-script-", ".js");

    temp_file.write(script)?;

    Ok(temp_file)
}

pub struct Scraper<'a> {
    arhiv: &'a Arhiv,
    chrome_bin_path: String,
    debug: bool,
}

impl<'a> Scraper<'a> {
    pub fn new(arhiv: &'a Arhiv) -> Result<Self> {
        let capabilities = EnvCapabilities::check()?;

        ensure!(capabilities.nodejs, "NodeJS must be available");
        let chrome_bin_path = capabilities
            .chrome
            .context("Chromium or Chrome must be available")?;

        Ok(Scraper {
            arhiv,
            chrome_bin_path,
            debug: false,
        })
    }

    pub fn debug(&mut self) {
        self.debug = true;
    }

    pub async fn scrape(&self, url: &str) -> Result<Vec<Document>> {
        log::info!("Scraping data from '{}'", url);

        let script_temp_file = get_script_temp_file()?;

        let mut child = Command::new("node")
            .arg("--title")
            .arg("arhiv-scraper")
            .arg(&script_temp_file.path)
            .arg(&self.chrome_bin_path)
            .arg(url)
            .arg(&self.debug.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn child process")?;

        let mut stdin = child
            .stdin
            .take()
            .context("Child did not have a handle to stdin")?;

        let stdout = child
            .stdout
            .take()
            .context("Child did not have a handle to stdout")?;

        let mut stderr = child
            .stderr
            .take()
            .context("Child did not have a handle to stderr")?;

        let stderr_handle = tokio::spawn(async move {
            let mut buffer = String::new();

            stderr.read_to_string(&mut buffer).await?;

            Ok::<String, Error>(buffer)
        });

        let child_handle = tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .context("Child process encountered an error")?;

            log::debug!("Child process status: {}", status);

            ensure!(
                status.success(),
                "Child process status isn't success: {}",
                status
            );

            Ok::<(), Error>(())
        });

        let mut tx = self.arhiv.get_tx()?;
        let mut documents = Vec::new();

        let mut reader = BufReader::new(stdout).lines();
        while let Some(line) = reader.next_line().await? {
            match self.execute_action(&line, &mut tx).await {
                Ok(document) => {
                    log::info!("Scraped document: {}", &document);

                    writeln(&mut stdin, &document.id).await?;

                    documents.push(document);
                }
                Err(err) => {
                    log::error!("Scraper action {} failed: {:?}", &line, err);

                    writeln(&mut stdin, "error").await?;
                }
            };
        }

        let (stderr_result, child_result) = try_join!(stderr_handle, child_handle)?;
        let stderr_logs = stderr_result.context("Child stderr failed")?;

        if !stderr_logs.is_empty() {
            log::error!("Child process stderr:\n{}", stderr_logs);
        }

        child_result.context("Child process failed")?;

        tx.commit()?;

        Ok(documents)
    }

    async fn execute_action(&self, action: &str, tx: &mut ArhivTransaction) -> Result<Document> {
        let action: ScraperAction =
            serde_json::from_str(action).context("Failed to parse ScraperAction")?;

        match action {
            ScraperAction::CreateAttachment { url } => {
                let download_result = Download::new(&url)?.start().await?;

                let attachment = Attachment::from_download_result(&download_result, tx)?;

                Ok(attachment.into())
            }
            ScraperAction::CreateDocument {
                document_type,
                data,
            } => {
                let mut document = Document::new_with_data(document_type, data);

                tx.stage_document(&mut document)?;

                Ok(document)
            }
        }
    }
}

async fn writeln(stdin: &mut ChildStdin, data: impl AsRef<str>) -> Result<()> {
    let mut data: Vec<u8> = data.as_ref().into();
    data.push(b'\n');

    stdin
        .write_all(&data)
        .await
        .context("failed to write response to stdin")?;

    Ok(())
}
