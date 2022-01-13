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
    clippy::cast_lossless,
    clippy::non_ascii_literal
)]

use std::process::Stdio;

use anyhow::{ensure, Context, Error, Result};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
    try_join,
};

use arhiv_core::{
    definitions::Attachment,
    entities::{Document, DocumentData},
    Arhiv,
};
use rs_utils::{download_file, log, EnvCapabilities, TempFile};

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

pub async fn scrape(
    arhiv: &Arhiv,
    capabilities: &EnvCapabilities,
    url: &str,
    debug: bool,
) -> Result<Vec<Document>> {
    ensure!(capabilities.nodejs, "NodeJS must be available");
    let chrome_bin_path = capabilities
        .chrome
        .as_ref()
        .context("Chromium or Chrome must be available")?;

    log::info!("Scraping data from '{}'", url);

    let script_temp_file = get_script_temp_file()?;

    let mut child = Command::new("node")
        .arg("--title")
        .arg("arhiv-scraper")
        .arg(&script_temp_file.path)
        .arg(chrome_bin_path)
        .arg(url)
        .arg(debug.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn child process")?;

    let mut stdin = child
        .stdin
        .take()
        .context("child did not have a handle to stdin")?;

    let stdout = child
        .stdout
        .take()
        .context("child did not have a handle to stdout")?;

    let mut stderr = child
        .stderr
        .take()
        .context("child did not have a handle to stderr")?;

    let stderr_handle = tokio::spawn(async move {
        let mut buffer = String::new();

        stderr.read_to_string(&mut buffer).await?;

        Ok::<String, Error>(buffer)
    });

    let child_handle = tokio::spawn(async move {
        let status = child
            .wait()
            .await
            .context("child process encountered an error")?;

        log::debug!("child process status: {}", status);

        ensure!(
            status.success(),
            "child process status isn't success: {}",
            status
        );

        Ok::<(), Error>(())
    });

    let mut tx = arhiv.get_tx()?;
    let mut documents = Vec::new();

    let mut reader = BufReader::new(stdout).lines();
    while let Some(line) = reader.next_line().await? {
        let action: ScraperAction =
            serde_json::from_str(&line).context("Failed to parse ScraperAction")?;

        match action {
            ScraperAction::CreateAttachment { url } => {
                // TODO remove downloaded file
                let file_path = download_file(&url).await?;

                let attachment = Attachment::create_tx(&file_path, true, arhiv, &mut tx)?;

                stdin
                    .write_all(&format!("{}\n", attachment.id).into_bytes())
                    .await
                    .context("failed to write response to stdin")?;

                documents.push(attachment.into());
            }
            ScraperAction::CreateDocument {
                document_type,
                data,
            } => {
                let mut document = Document::new_with_data(document_type, data);

                log::info!("Scraped document:\n{}", &document);

                arhiv.tx_stage_document(&mut document, &mut tx)?;

                stdin
                    .write_all(&format!("{}\n", document.id).into_bytes())
                    .await
                    .context("failed to write response to stdin")?;

                documents.push(document);
            }
        };
    }

    let (stderr_result, child_result) = try_join!(stderr_handle, child_handle)?;
    let stderr_logs = stderr_result.context("stderr failed")?;

    child_result.with_context(|| format!("child process failed:\n{}", stderr_logs))?;

    tx.commit()?;

    Ok(documents)
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use serde_json::Value;

    use arhiv_core::test_arhiv::TestArhiv;
    use rs_utils::EnvCapabilities;

    use super::scrape;

    async fn scrape_and_extract(
        url: &str,
        expected_document_type: &str,
        expected_documents_count: usize,
    ) -> Result<Value> {
        let arhiv = TestArhiv::new_prime();

        let capabilities = EnvCapabilities::must_check();

        let documents = scrape(&arhiv, &capabilities, url, false).await?;

        assert_eq!(documents.len(), expected_documents_count);

        let document = documents
            .into_iter()
            .find(|document| document.document_type == expected_document_type)
            .ok_or_else(|| anyhow!("can't find document with type {}", expected_document_type))?;

        Ok(document.data.into())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_yakaboo_book() -> Result<()> {
        let result = scrape_and_extract(
            "https://www.yakaboo.ua/ua/stories-of-your-life-and-others.html",
            "book",
            2,
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_film() -> Result<()> {
        let result = scrape_and_extract("https://www.imdb.com/title/tt0133093/", "film", 2).await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_steam_game() -> Result<()> {
        let result = scrape_and_extract(
            "https://store.steampowered.com/app/814380/Sekiro_Shadows_Die_Twice__GOTY_Edition/",
            "game",
            2,
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }
}
