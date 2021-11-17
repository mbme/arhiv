use std::env;
use std::fs;
use std::os::unix::prelude::MetadataExt;

use anyhow::*;
use tokio::fs as tokio_fs;
use tokio_util::codec::{BytesCodec, FramedRead};

pub fn path_exists(path: impl AsRef<str>) -> bool {
    fs::metadata(path.as_ref()).is_ok()
}

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
    ensure!(dir_exists(path)?, "dir doesn't exist {}", path);

    Ok(())
}

pub fn ensure_file_exists(path: &str) -> Result<()> {
    ensure!(file_exists(path)?, "file doesn't exist {}", path);

    Ok(())
}

/// check if path1 and path2 belong to the same filesystem or not
pub fn is_same_filesystem(path1: &str, path2: &str) -> Result<bool> {
    let meta1 = fs::metadata(path1)?;
    let meta2 = fs::metadata(path2)?;

    Ok(meta1.dev() == meta2.dev())
}

#[must_use]
pub fn get_file_name(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .expect("file must have name")
        .to_str()
        .expect("file name must be valid string")
}

pub async fn read_file_as_stream(path: &str) -> Result<FramedRead<tokio_fs::File, BytesCodec>> {
    let file = tokio_fs::File::open(path).await?;

    Ok(FramedRead::new(file, BytesCodec::new()))
}

#[must_use]
pub fn get_home_dir() -> Option<String> {
    env::var_os("HOME").map(|path| {
        path.into_string()
            .expect("HOME env var must be a valid string")
    })
}

/// `$XDG_CONFIG_HOME` or `$HOME/.config`
#[must_use]
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

/// `$XDG_DOWNLOAD_DIR` or `$HOME/Downloads`
#[must_use]
pub fn get_downloads_dir() -> Option<String> {
    if let Some(path) = env::var_os("XDG_DOWNLOAD_DIR") {
        return path
            .into_string()
            .expect("XDG_DOWNLOAD_DIR env var must be a valid string")
            .into();
    }

    if let Some(path) = env::var_os("HOME") {
        return format!(
            "{}/Downloads",
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

pub fn move_file(src: impl AsRef<str>, dest: impl AsRef<str>) -> Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    let err = {
        if let Err(err) = fs::rename(src, dest) {
            err
        } else {
            return Ok(());
        }
    };

    // check for Invalid cross-device link (os error 18)
    if err.raw_os_error() != Some(18) {
        bail!(err);
    }

    // if error is due to src and dest being on different file systems
    // then copy src into dest, and remove src

    fs::copy(src, dest)?;

    if let Err(err) = fs::remove_file(src) {
        log::warn!("Failed to remove source file {}: {}", src, err);
    }

    Ok(())
}

pub fn into_absolute_path(path: impl AsRef<str>) -> Result<String> {
    let path = shellexpand::full(path.as_ref()).context("failed to expand path")?;

    let path = fs::canonicalize(path.as_ref()).context("failed to canonicalize path")?;

    path.into_os_string().into_string().map_err(|err| {
        anyhow!(
            "failed to convert path to string: {}",
            err.to_string_lossy()
        )
    })
}
