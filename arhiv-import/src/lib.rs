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

use anyhow::{bail, ensure, Context, Error, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    try_join,
};

use arhiv_core::{
    definitions::Attachment,
    entities::{Document, DocumentData},
    Arhiv,
};
use rs_utils::{download_file, log, TempFile};

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ImporterAction {
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

    let temp_file = TempFile::new_with_details("arhiv-import-script-", ".js");

    temp_file.write(script)?;

    Ok(temp_file)
}

pub async fn scrape(arhiv: &Arhiv, url: &str, debug: bool, confirm: bool) -> Result<Vec<Document>> {
    log::info!("Scraping data from '{}'", url);

    let script_temp_file = get_script_temp_file()?;

    let mut child = Command::new("node")
        .arg("--title")
        .arg("arhiv-import-scraper")
        .arg(&script_temp_file.path)
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

    let stderr = child
        .stderr
        .take()
        .context("child did not have a handle to stderr")?;

    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .context("failed to read next line")?
        {
            log::warn!("importers: {}", line);
        }

        Ok::<(), Error>(())
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
        let action: ImporterAction =
            serde_json::from_str(&line).context("Failed to parse ImporterAction")?;

        match action {
            ImporterAction::CreateAttachment { url } => {
                let file_path = download_file(&url).await?;

                let attachment = Attachment::create_tx(&file_path, true, arhiv, &mut tx)?;

                stdin
                    .write_all(&format!("{}\n", attachment.id).into_bytes())
                    .await
                    .context("failed to write response to stdin")?;

                documents.push(attachment.into());
            }
            ImporterAction::CreateDocument {
                document_type,
                data,
            } => {
                let mut document = Document::new_with_data(document_type, data);

                log::info!("Scraped document:\n{}", &document);
                confirm_if_needed(confirm).await?;

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
    stderr_result.context("stderr failed")?;
    child_result.context("child process failed")?;

    tx.commit()?;

    Ok(documents)
}

async fn confirm_if_needed(confirm: bool) -> Result<()> {
    if !confirm {
        return Ok(());
    }

    let proceed = tokio::task::spawn_blocking(|| {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you really want to continue?")
            .default(true)
            .interact()
            .context("failed to ask confirmation")
    })
    .await??;

    if proceed {
        return Ok(());
    }

    bail!("confirmation failed")
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use serde_json::Value;

    use arhiv_core::test_arhiv::TestArhiv;

    use super::scrape;

    async fn scrape_and_extract(
        url: &str,
        expected_document_type: &str,
        expected_documents_count: usize,
    ) -> Result<Value> {
        let arhiv = TestArhiv::new_prime();

        let documents = scrape(&arhiv, url, false, false).await?;

        assert_eq!(documents.len(), expected_documents_count);

        let document = documents
            .into_iter()
            .find(|document| document.document_type == expected_document_type)
            .ok_or(anyhow!(
                "can't find document with type {}",
                expected_document_type
            ))?;

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
}
