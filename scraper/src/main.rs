#![deny(clippy::all)]

use clap::Parser;

use rs_utils::log;

use scraper::Scraper;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
    /// URL to scrape
    #[clap(required = true, value_parser)]
    url: String,

    /// Open browser and allow to run scripts manually
    #[clap(long, action)]
    manual: bool,

    #[clap(long, action)]
    debug: bool,

    /// Emulate mobile device
    #[clap(long, action)]
    mobile: bool,
}

pub fn main() {
    log::setup_logger();

    let args = Args::parse();

    let mut scraper = Scraper::new().expect("failed to init scraper");

    if args.debug {
        scraper.debug();
    }

    if args.mobile {
        scraper.emulate_mobile();
    }

    if args.manual {
        scraper.manual();
    }

    let result = scraper.scrape(&args.url).expect("failed to scrape");

    println!(
        "{}",
        serde_json::to_string_pretty(&result).expect("must serialize")
    );
}
