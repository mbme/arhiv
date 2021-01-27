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

// recursively search from current dir upwards for {file_name}
pub fn locate_dominating_file<S: Into<String>>(file_name: S) -> Result<String> {
    let file_name = file_name.into();

    let mut dir = env::current_dir().context("must be able to get current dir")?;

    loop {
        let candidate = format!(
            "{}/{}",
            &dir.to_str().expect("must be able to serialize path"),
            file_name,
        );

        if file_exists(&candidate).unwrap_or(false) {
            return Ok(candidate);
        }

        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            bail!("Can't locate dominating file {}", file_name);
        }
    }
}
