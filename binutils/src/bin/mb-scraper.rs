use std::env;

use clap::{Parser, ValueHint};
use serde::{Deserialize, Serialize};

use rs_utils::{get_crate_version, log};

use scraper::ScraperOptions;

/// Extract data from websites and output it in JSON format
#[derive(Parser, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[clap(version = get_crate_version(), about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// URL to scrape
    #[clap(required = true, value_parser, value_hint = ValueHint::Url)]
    url: String,

    /// Open browser and allow to run scripts manually
    #[serde(default)]
    #[clap(long, action)]
    manual: bool,

    #[serde(default)]
    #[clap(long, action)]
    debug: bool,

    /// Emulate mobile device
    #[serde(default)]
    #[clap(long, action)]
    mobile: bool,

    /// Save a screenshot of the page in a PNG format into this file
    #[serde(default)]
    #[clap(long, action)]
    screenshot_file: Option<String>,
}

#[tokio::main]
pub async fn main() {
    log::setup_logger();

    // TODO more feature similar to https://shot-scraper.datasette.io/en/stable/

    let args = if env::var("JSON_ARG_MOODE").is_ok() {
        let args: Vec<String> = env::args().collect();
        let arg = args
            .get(1)
            .expect("argument must be provided in JSON_ARG_MOODE");

        serde_json::from_str(arg).expect("invalid JSON_ARG_MOODE argument")
    } else {
        Args::parse()
    };

    let result = ScraperOptions {
        debug: args.debug,
        emulate_mobile: args.mobile,
        manual: args.manual,
        screenshot_file: args.screenshot_file,
    }
    .scrape(&args.url)
    .await
    .expect("failed to scrape");

    println!(
        "{}",
        serde_json::to_string_pretty(&result).expect("must serialize")
    );
}
