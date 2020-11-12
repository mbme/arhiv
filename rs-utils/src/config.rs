use crate::fs::file_exists;
use anyhow::*;
use std::env;

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

// In development, recursively search from current dir upwards for {file_name}
// In production, look up {file_name} in a system config directory
pub fn find_config_file<S: Into<String>>(file_name: S) -> Result<String> {
    let file_name = file_name.into();

    if cfg!(feature = "production-mode") {
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
