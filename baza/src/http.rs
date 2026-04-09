use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use std::sync::LazyLock;
use url::Url;

use baza_common::is_image_path;

pub fn parse_url(url: &str) -> Result<Url> {
    Url::parse(url).context("Failed to parse url")
}

#[must_use]
pub fn extract_file_name_from_url(url: &Url) -> String {
    let file_name = url
        .path_segments()
        .and_then(Iterator::last)
        .map(ToString::to_string)
        .unwrap_or_default();

    if file_name.is_empty() {
        return "unknown".to_string();
    }

    file_name
}

pub fn is_http_url(url: &Url) -> bool {
    url.scheme() == "http" || url.scheme() == "https"
}

pub fn is_image_url(url: &Url) -> bool {
    is_image_path(extract_file_name_from_url(url))
}

pub fn parse_content_disposition_header(header: &str) -> Result<Option<String>> {
    if header == "inline" || header == "attachment" {
        return Ok(None);
    }

    let filename = if let Some(values) = header.split_once('=') {
        values.1
    } else {
        bail!("Failed to parse Content-Disposition header: {header}");
    };

    let filename = if let Some((_, filename)) = filename.split_once('"') {
        filename
    } else if let Some((_, filename)) = filename.split_once("''") {
        filename
    } else {
        bail!("Failed to parse Content-Disposition header: {header}");
    };

    Ok(Some(filename.trim_end_matches('"').to_string()))
}

pub fn parse_content_type_header(header: &str) -> Result<(String, Option<String>, Option<String>)> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^([[:alnum:]-]+/[[:alnum:]-]+)(?:; (charset|boundary)=(.+))?$")
            .expect("failed to create Content-Type regex")
    });

    let captures = RE
        .captures(header)
        .with_context(|| format!("Unsupported Content-Type header: {header}"))?;

    let content_type = captures
        .get(1)
        .context("failed to capture type group")?
        .as_str()
        .to_lowercase();

    if let Some(prop) = captures.get(2) {
        let value = captures
            .get(3)
            .context("failed to capture value group")?
            .as_str()
            .to_string();

        match prop.as_str() {
            "charset" => return Ok((content_type, Some(value), None)),
            "boundary" => return Ok((content_type, None, Some(value))),
            other => bail!("Unexpected property: {other}"),
        };
    }

    Ok((content_type, None, None))
}

pub fn parse_content_range_header(header: &str) -> Result<(u64, u64, u64)> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^bytes (\d+)-(\d+)/(\d+)$").expect("failed to create Content-Range regex")
    });

    let captures = RE
        .captures(header)
        .with_context(|| format!("Unsupported Content-Range header: {header}"))?;

    let start = captures
        .get(1)
        .context("failed to capture range start group")?
        .as_str()
        .parse()
        .context("failed to parse range start group")?;

    let end = captures
        .get(2)
        .context("failed to capture range end group")?
        .as_str()
        .parse()
        .context("failed to parse range end group")?;

    let total_size = captures
        .get(3)
        .context("failed to capture total size group")?
        .as_str()
        .parse()
        .context("failed to parse total size group")?;

    ensure!(
        start < end,
        "Content-Range start {start} must be smaller than end {end}"
    );
    ensure!(
        end < total_size,
        "Content-Range end {end} must be smaller than end {total_size}"
    );

    Ok((start, end, total_size))
}
