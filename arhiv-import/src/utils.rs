use anyhow::*;
use dialoguer::{theme::ColorfulTheme, Confirm};

use rs_utils::run_js_script;

pub fn scrape(url: &str) -> Result<String> {
    let script = include_str!("../dist/bundle.js");

    run_js_script(script, vec![url])
}

pub fn confirm_if_needed(confirm: bool) -> Result<bool> {
    if !confirm {
        return Ok(true);
    }

    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you really want to continue?")
        .default(true)
        .interact()
        .context("failed to ask confirmation")
}
