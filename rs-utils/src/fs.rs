use std::{
    env, fs,
    io::ErrorKind,
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, ensure, Context, Result};

use crate::{bytes_to_hex_string, get_file_hash_blake3, get_string_hash_blake3};

pub fn path_exists(path: impl AsRef<str>) -> bool {
    fs::metadata(path.as_ref()).is_ok()
}

pub fn path_to_string(path: impl Into<PathBuf>) -> Result<String> {
    let result = path
        .into()
        .to_str()
        .context("Failed to convert file path to string")?
        .to_string();

    Ok(result)
}

/// This won't follow symlinks
pub fn file_exists(path: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() => Err(anyhow!("path isn't a file: {}", path)),

        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

/// This won't follow symlinks
pub fn dir_exists(path: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_dir() => Err(anyhow!("path isn't a directory: {}", path)),

        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

pub fn ensure_dir_exists(path: &str) -> Result<()> {
    ensure!(dir_exists(path)?, "dir doesn't exist {}", path);

    Ok(())
}

pub fn is_empty_dir(path: &str) -> Result<bool> {
    Ok(fs::read_dir(path)?.next().is_none())
}

pub fn ensure_file_exists(path: &str) -> Result<()> {
    ensure!(file_exists(path)?, "file doesn't exist {}", path);

    Ok(())
}

pub fn get_symlink_target_path(path: &str) -> Result<String> {
    let target_path = fs::read_link(path).context("failed to read a symlink")?;

    path_to_string(target_path)
}

/// check if path1 and path2 belong to the same filesystem or not
pub fn is_same_filesystem(path1: &str, path2: &str) -> Result<bool> {
    let meta1 = fs::metadata(path1)?;
    let meta2 = fs::metadata(path2)?;

    Ok(meta1.dev() == meta2.dev())
}

#[must_use]
pub fn get_file_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .expect("file must have name")
        .to_str()
        .expect("file name must be valid string")
}

pub fn get_file_size(path: &str) -> Result<u64> {
    let metadata = fs::metadata(path).context("failed to read file metadata")?;

    Ok(metadata.len())
}

#[must_use]
pub fn get_file_extension(path: &str) -> Option<String> {
    let extension = Path::new(path).extension()?.to_str()?.to_string();

    Some(extension)
}

pub fn remove_file_extension(path: &str) -> Result<String> {
    let filename = Path::new(path)
        .file_stem()
        .context("failed to remove file extension")?;

    path_to_string(filename)
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

    match fs::rename(src, dest) {
        Err(err) if err.raw_os_error() == Some(18) => {
            // check for Invalid cross-device link (os error 18)
        }
        Err(err) => {
            return Err(err).context("failed to rename file");
        }
        Ok(_) => {
            return Ok(());
        }
    };

    // if error is due to src and dest being on different file systems
    // then copy src into dest, and remove src

    fs::copy(src, dest).context("failed to copy file data")?;

    if let Err(err) = fs::remove_file(src) {
        log::warn!("Failed to remove source file {}: {}", src, err);
    }

    Ok(())
}

pub fn into_absolute_path(path: impl AsRef<str>, canonicalize: bool) -> Result<String> {
    let path = shellexpand::full(path.as_ref()).context("failed to expand path")?;

    let path = path.as_ref();

    let mut path = if path.starts_with('/') {
        PathBuf::from(path)
    } else {
        current_dir_relpath(path)
    };

    // NOTE: path must exist
    if canonicalize {
        path = path
            .canonicalize()
            .with_context(|| format!("failed to canonicalize path {}", path.to_string_lossy()))?;
    }

    path_to_string(path)
}

#[must_use]
pub fn is_readable(metadata: &fs::Metadata) -> bool {
    let mode = metadata.mode();

    // TODO check also user / group (uid/gid)

    let user_has_read_access = mode & 0o400 != 0;
    let user_has_exec_access = mode & 0o100 != 0;

    if metadata.is_dir() {
        return user_has_read_access && user_has_exec_access;
    }

    user_has_read_access
}

pub fn get_media_type(file_path: impl AsRef<str>) -> Result<String> {
    let file_path = file_path.as_ref();

    if let Some(kind) = infer::get_from_path(file_path).context("failed to infer media type")? {
        return Ok(kind.mime_type().to_string());
    }

    Ok("application/octet-stream".to_string())
}

pub fn infer_extension_by_file_mime_type(file_path: &str) -> Result<Option<String>> {
    if let Some(kind) = infer::get_from_path(file_path).context("failed to infer mime type")? {
        return Ok(Some(kind.extension().to_string()));
    }

    Ok(None)
}

#[must_use]
pub fn get_extension_for_mime_string(mime: &str) -> Option<String> {
    let extensions = mime_guess::get_mime_extensions_str(mime).unwrap_or_default();

    Some((*extensions.first()?).to_string())
}

#[must_use]
pub fn get_mime_from_path(path: impl AsRef<str>) -> String {
    mime_guess::from_path(path.as_ref())
        .first_or_octet_stream()
        .to_string()
}

pub fn get_dir_checksum(path: impl AsRef<str>) -> Result<String> {
    let mut items: Vec<(String, String)> = Vec::new();

    for entry in fs::read_dir(path.as_ref())? {
        let entry = entry?;

        let name = path_to_string(entry.file_name())?;

        let path = path_to_string(entry.path())?;

        let hash = if fs::metadata(&path)?.is_dir() {
            get_dir_checksum(&path)?
        } else {
            bytes_to_hex_string(&get_file_hash_blake3(&path)?)
        };

        items.push((name, hash));
    }

    // sort by name
    items.sort_by(|a, b| a.0.cmp(&b.0));

    let result: String = items
        .into_iter()
        .flat_map(|item| vec![item.0, item.1])
        .collect();

    Ok(get_string_hash_blake3(&result))
}

pub fn create_dir_if_not_exist(dir_path: impl Into<PathBuf>) -> Result<()> {
    let dir_path = dir_path.into();

    if !dir_path.exists() {
        fs::create_dir(&dir_path).context(anyhow!("failed to create dir {:?}", dir_path))?;
    }

    Ok(())
}

pub fn create_file_if_not_exist(file_path: impl Into<PathBuf>) -> Result<()> {
    let file_path = file_path.into();

    if !file_path.exists() {
        fs::File::create(&file_path).context(anyhow!("failed to create file {:?}", file_path))?;
    }

    Ok(())
}

pub fn remove_file_if_exists(file_path: &str) -> Result<()> {
    match fs::remove_file(file_path) {
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).context("failed to remove file");
        }
        Ok(_) => {}
    }

    Ok(())
}

#[must_use]
pub fn workspace_relpath(subpath: &str) -> String {
    // Here CARGO_MANIFEST_DIR is /typed-v/rs-utils/
    format!("{}/../{}", env!("CARGO_MANIFEST_DIR"), subpath)
}

#[must_use]
pub fn current_dir_relpath(subpath: &str) -> PathBuf {
    let mut resource = env::current_dir().expect("invalid current directory");

    resource.push(subpath);

    resource
}

#[must_use]
pub fn is_image_filename(filename: impl AsRef<str>) -> bool {
    let ext = filename.as_ref().rsplit('.').next().unwrap_or_default();

    ext.eq_ignore_ascii_case("png")
        || ext.eq_ignore_ascii_case("jpg")
        || ext.eq_ignore_ascii_case("jpeg")
        || ext.eq_ignore_ascii_case("svg")
}

#[cfg(test)]
mod tests {
    use crate::workspace_relpath;

    use super::*;

    #[test]
    fn test_get_media_type() {
        assert_eq!(
            get_media_type(workspace_relpath("resources/text.txt")).unwrap(),
            "application/octet-stream"
        );
        assert_eq!(
            get_media_type(workspace_relpath("resources/k2.jpg")).unwrap(),
            "image/jpeg"
        );
        assert_eq!(
            get_media_type(workspace_relpath("resources/favicon-16x16.png")).unwrap(),
            "image/png"
        );
    }
}
