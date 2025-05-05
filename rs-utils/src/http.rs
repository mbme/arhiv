use std::{io::Read, sync::LazyLock};

use anyhow::{Context, Result, bail, ensure};
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use regex::Regex;
use tokio::{
    fs as tokio_fs,
    io::{AsyncReadExt, AsyncSeekExt},
};
use tokio_util::codec::{BytesCodec, FramedRead};
use url::Url;

use crate::{is_image_path, reader_to_stream};

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
    let file_name = extract_file_name_from_url(url);

    is_image_path(file_name)
}

pub fn parse_content_disposition_header(header: &str) -> Result<Option<String>> {
    if header == "inline" || header == "attachment" {
        return Ok(None);
    }

    let filename = if let Some(values) = header.split_once("=") {
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
            other => bail!("Unexpected property: {}", other),
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
        "Content-Range start {} must be smaller than end {}",
        start,
        end
    );

    ensure!(
        end < total_size,
        "Content-Range end {} must be smaller than end {}",
        start,
        total_size
    );

    Ok((start, end, total_size))
}

pub fn parse_range_header(header: &str) -> Result<(u64, Option<u64>)> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^bytes=(\d+)-(\d+)?$").expect("failed to create Range regex")
    });

    let captures = RE
        .captures(header)
        .with_context(|| format!("Unsupported Range header: {header}"))?;

    let start: u64 = captures
        .get(1)
        .context("failed to capture range start group")?
        .as_str()
        .parse()
        .context("failed to parse range start group")?;

    let end = captures
        .get(2)
        .map(|capture| capture.as_str().parse::<u64>())
        .transpose()
        .context("failed to parse range end group")?;

    Ok((start, end))
}

pub async fn create_body_from_file(
    path: &str,
    start_pos: u64,
    limit: Option<u64>,
) -> Result<Response> {
    let mut file = tokio_fs::File::open(path).await?;

    let size = file.metadata().await?.len();
    ensure!(
        start_pos < size - 1,
        "start_pos must be less than file size {} - 1",
        size,
    );

    file.seek(std::io::SeekFrom::Start(start_pos)).await?;

    let body = if let Some(limit) = limit {
        ensure!(limit > 0, "limit {} must be greater than 0", limit);

        ensure!(
            start_pos + limit <= size,
            "start_pos {} + limit {} must be <= file size {}",
            start_pos,
            limit,
            size
        );

        let stream = FramedRead::new(file.take(limit), BytesCodec::new());
        Body::from_stream(stream).into_response()
    } else {
        let stream = FramedRead::new(file, BytesCodec::new());
        Body::from_stream(stream).into_response()
    };

    Ok(body)
}

pub async fn create_body_from_reader<R: Read + Send + 'static>(
    reader: R,
    limit: Option<u64>,
) -> Result<Body> {
    let s = reader_to_stream(reader, 1024 * 1024);

    let body = if let Some(limit) = limit {
        let s = s.take(limit as usize);

        Body::from_stream(s)
    } else {
        Body::from_stream(s)
    };

    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use axum::body::to_bytes;

    use crate::workspace_relpath;

    use super::*;

    #[test]
    fn test_extract_file_name_from_url() {
        assert_eq!(
            extract_file_name_from_url(&Url::parse("http://test.com/test").unwrap()),
            "test"
        );
        assert_eq!(
            extract_file_name_from_url(&Url::parse("http://test.com/123/test").unwrap()),
            "test"
        );
        assert_eq!(
            extract_file_name_from_url(&Url::parse("http://test.com/").unwrap()),
            "unknown"
        );
        assert_eq!(
            extract_file_name_from_url(&Url::parse("http://test.com/image.png?test=123").unwrap()),
            "image.png"
        );
    }

    #[test]
    fn test_parse_content_range_header() {
        assert_eq!(
            parse_content_range_header("bytes 1-2/3").unwrap(),
            (1, 2, 3)
        );

        assert_eq!(
            parse_content_range_header("bytes 10-20000/300000").unwrap(),
            (10, 20000, 300_000)
        );

        assert!(parse_content_range_header("bytes 1-2/*").is_err());

        assert!(parse_content_range_header("bytes */*").is_err());
    }

    #[test]
    fn test_parse_content_disposition_header() {
        assert_eq!(parse_content_disposition_header("inline").unwrap(), None);
        assert_eq!(
            parse_content_disposition_header("attachment").unwrap(),
            None
        );
        assert_eq!(
            parse_content_disposition_header("attachment; filename=\"test\"").unwrap(),
            Some("test".to_string())
        );
        assert_eq!(
            parse_content_disposition_header(
                "inline;filename*=UTF-8''Olha_Kharlan_2014_European_Championships_SFS-EQ_t164825.jpg"
            )
            .unwrap(),
            Some("Olha_Kharlan_2014_European_Championships_SFS-EQ_t164825.jpg".to_string())
        );

        assert!(parse_content_disposition_header("wrong").is_err());
    }

    #[test]
    fn test_parse_content_type_header() {
        assert_eq!(
            parse_content_type_header("text/html; charset=UTF-8").unwrap(),
            ("text/html".to_string(), Some("UTF-8".to_string()), None)
        );

        assert_eq!(
            parse_content_type_header("multipart/form-data; boundary=something").unwrap(),
            (
                "multipart/form-data".to_string(),
                None,
                Some("something".to_string())
            )
        );

        assert_eq!(
            parse_content_type_header("text/html").unwrap(),
            ("text/html".to_string(), None, None)
        );

        assert_eq!(
            parse_content_type_header("TEXT/HTML").unwrap(),
            ("text/html".to_string(), None, None)
        );
    }

    #[test]
    fn test_parse_range_header() {
        assert_eq!(parse_range_header("bytes=0-100").unwrap(), (0, Some(100)));
        assert_eq!(parse_range_header("bytes=0-").unwrap(), (0, None));
    }

    #[tokio::test]
    async fn test_create_body_from_file() -> Result<()> {
        let file_path = workspace_relpath("resources/k2.jpg");
        let orig_data = fs::read(&file_path)?;

        {
            let response = create_body_from_file(&file_path, 0, None).await?;
            let data = to_bytes(response.into_body(), 999999999999).await?;

            assert_eq!(&orig_data, &data);
        }

        {
            let response = create_body_from_file(&file_path, 10, None).await?;
            let data = to_bytes(response.into_body(), 99999999999).await?;

            assert_eq!(&orig_data[10..], &data);
        }

        {
            let response = create_body_from_file(&file_path, 10, Some(10)).await?;
            let data = to_bytes(response.into_body(), 999999999999).await?;

            assert_eq!(data.len(), 10);
            assert_eq!(&orig_data[10..20], &data);
        }

        {
            let result = create_body_from_file(&file_path, 2_000_000_000, None).await;

            assert!(result.is_err());
        }

        {
            let result = create_body_from_file(&file_path, 0, Some(0)).await;

            assert!(result.is_err());
        }

        {
            let result = create_body_from_file(&file_path, 0, Some(2_000_000_000)).await;

            assert!(result.is_err());
        }

        Ok(())
    }
}
