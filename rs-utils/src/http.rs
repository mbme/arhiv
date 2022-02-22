use anyhow::{bail, ensure, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

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

pub fn parse_content_disposition_header(header: &str) -> Result<Option<String>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^attachment; filename="(.+)"$"#)
            .expect("failed to create Content-Disposition regex");
    }

    if header == "inline" || header == "attachment" {
        return Ok(None);
    }

    let captures = RE
        .captures(header)
        .with_context(|| format!("Unsupported Content-Disposition header: {}", header))?;

    let filename = captures
        .get(1)
        .context("failed to capture filename group")?
        .as_str()
        .to_string();

    if filename.is_empty() {
        Ok(None)
    } else {
        Ok(Some(filename))
    }
}

pub fn parse_content_type_header(header: &str) -> Result<(String, Option<String>, Option<String>)> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^([[:alnum:]-]+/[[:alnum:]-]+)(?:; (charset|boundary)=(.+))?$")
                .expect("failed to create Content-Type regex");
    }

    let captures = RE
        .captures(header)
        .with_context(|| format!("Unsupported Content-Type header: {}", header))?;

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
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^bytes (\d+)-(\d+)/(\d+)$").expect("failed to create Content-Range regex");
    }

    let captures = RE
        .captures(header)
        .with_context(|| format!("Unsupported Content-Range header: {}", header))?;

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

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn test_parse_content_range_header() {
        assert_eq!(
            parse_content_range_header("bytes 1-2/3").unwrap(),
            (1, 2, 3)
        );

        assert_eq!(
            parse_content_range_header("bytes 10-20000/300000").unwrap(),
            (10, 20000, 300000)
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

        // no double quotes in file names
        assert!(parse_content_disposition_header(r#"attachment; filename="test\"1"#).is_err());

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
}
