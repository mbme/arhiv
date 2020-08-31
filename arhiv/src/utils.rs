use anyhow::*;
use std::env;
use std::fs;
use tokio::fs as tokio_fs;
use tokio_util::codec::{BytesCodec, FramedRead};

pub fn file_exists(path: &str) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => Err(anyhow!("path isn't a file: {}", path)),

        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

pub fn dir_exists(path: &str) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_dir() => Err(anyhow!("path isn't a directory: {}", path)),

        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

pub fn ensure_dir_exists(path: &str) -> Result<()> {
    if dir_exists(path)? {
        Ok(())
    } else {
        Err(anyhow!("dir doesn't exist {}", path))
    }
}

pub fn ensure_file_exists(path: &str) -> Result<()> {
    if file_exists(path)? {
        Ok(())
    } else {
        Err(anyhow!("file doesn't exist {}", path))
    }
}

pub async fn read_file_as_stream(path: &str) -> Result<FramedRead<tokio_fs::File, BytesCodec>> {
    let file = tokio_fs::File::open(path).await?;

    Ok(FramedRead::new(file, BytesCodec::new()))
}

pub fn project_relpath(subpath: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), subpath)
}

// $XDG_CONFIG_HOME or $HOME/.config
pub fn get_config_home() -> Option<String> {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return path
            .into_string()
            .expect("XDG_CONFIG_HOME env var must be a valid string")
            .into();
    }

    if let Some(path) = env::var_os("HOME") {
        return format!(
            "{}/.config",
            path.into_string()
                .expect("HOME env var must be a valid string")
        )
        .into();
    }

    None
}

// development or production
const MODE: Option<&'static str> = option_env!("MODE");

pub fn is_production_mode() -> bool {
    MODE.unwrap_or("development") == "production"
}

// In development, recursively search from current dir upwards for {file_name}
// In production, look up {file_name} in a system config directory
pub fn find_config_file<S: Into<String>>(file_name: S) -> Result<String> {
    let file_name = file_name.into();

    if is_production_mode() {
        let config_home = get_config_home().ok_or(anyhow!("Failed to find user config dir"))?;
        let config = format!("{}/{}", config_home, file_name);

        if file_exists(&config).unwrap_or(false) {
            return Ok(config);
        }

        bail!("Can't find Arhiv config at {}", config);
    }

    // in development

    let mut dir = env::current_dir().context("must be able to get current dir")?;

    loop {
        let config = format!(
            "{}/{}",
            &dir.to_str().expect("must be able to serialize path"),
            file_name,
        );

        if file_exists(&config).unwrap_or(false) {
            return Ok(config);
        }

        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            bail!("Can't find arhiv config");
        }
    }
}

pub fn fuzzy_match(needle: &str, haystack: &str) -> bool {
    // if needle is empty then it matches everything
    if needle.is_empty() {
        return true;
    }

    if needle.len() > haystack.len() {
        return false;
    }

    let needle = needle.to_lowercase();
    let haystack = haystack.to_lowercase();

    if needle.len() == haystack.len() {
        return needle == haystack;
    }

    let mut haystack_chars = haystack.chars();

    'outer: for needle_char in needle.chars() {
        loop {
            if let Some(haystack_char) = haystack_chars.next() {
                if haystack_char == needle_char {
                    continue 'outer;
                }
            } else {
                return false;
            }
        }
    }

    return true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert_eq!(fuzzy_match("", ""), true);
        assert_eq!(fuzzy_match("", "test"), true);
        assert_eq!(fuzzy_match("test", "test"), true);
        assert_eq!(fuzzy_match("test", "te"), false);
        assert_eq!(fuzzy_match("TEST", "teSt"), true);
        assert_eq!(fuzzy_match("123", "1test2test3"), true);
        assert_eq!(fuzzy_match("123", "123test2test3"), true);
        assert_eq!(fuzzy_match("123", "12test2test"), false);
    }
}
