#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use clap::{crate_version, App, AppSettings, Arg};

use arhiv_core::definitions::get_standard_schema;

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build_app() -> App<'static> {
    App::new("arhiv")
        .bin_name("arhiv")
        .subcommand(
            App::new("init")
                .about("Initialize Arhiv instance on local machine")
                .arg(
                    Arg::new("arhiv_id")
                        .long("arhiv_id")
                        .required(true)
                        .index(1)
                        .help("Arhiv id to use"),
                )
                .arg(
                    Arg::new("prime")
                        .long("prime")
                        .display_order(1)
                        .help("Initialize prime instance"),
                ),
        )
        .subcommand(
            App::new("sync") //
                .about("Sync changes"),
        )
        .subcommand(App::new("apply-migrations").about("Upgrade arhiv db schema to latest version"))
        .subcommand(
            App::new("backup") //
                .about("Backup arhiv data"),
        )
        .subcommand(
            App::new("ui-server") //
                .about("Run arhiv UI server"),
        )
        .subcommand(
            App::new("ui-open") //
                .about("Open document in UI")
                .arg(
                    Arg::new("id")
                        .index(1)
                        .required(true)
                        .help("document id to open"),
                )
                .arg(
                    Arg::new("browser")
                        .long("browser")
                        .takes_value(true)
                        .min_values(0)
                        .env("BROWSER")
                        .help("Open using provided browser or fall back to $BROWSER env variable"),
                ),
        )
        .subcommand(
            App::new("prime-server").about("Run prime server").arg(
                Arg::new("port")
                    .long("port")
                    .takes_value(true)
                    .default_value("23420")
                    .help("Listen on specific port"),
            ),
        )
        .subcommand(
            App::new("status") //
                .about("Print current status"),
        )
        .subcommand(
            App::new("config") //
                .about("Print config")
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .display_order(1)
                        .help("Prints config template"),
                ),
        )
        .subcommand(
            App::new("get")
                .about("Get document by id")
                .arg(Arg::new("id").required(true).help("id of the document")),
        )
        .subcommand(
            App::new("add")
                .about("Add new document")
                .arg(
                    Arg::new("document_type")
                        .required(true)
                        .possible_values(get_standard_schema().get_document_types(true))
                        .index(1)
                        .help("One of known document types"),
                )
                .arg(
                    Arg::new("data")
                        .required(true)
                        .index(2)
                        .help("JSON object with document props"),
                ),
        )
        .subcommand(
            App::new("attach")
                .about("Add new attachment. Will hard link or copy file to arhiv.")
                .arg(
                    Arg::new("file_path")
                        .required(true)
                        .index(1)
                        .help("Absolute path to file to save"),
                )
                .arg(Arg::new("move_file").short('m').help("Move file to arhiv")),
        )
        .subcommand(
            App::new("scrape")
                .about("Scrape data and create document")
                .arg(
                    Arg::new("url") //
                        .required(true)
                        .index(1)
                        .help("url to scrape"),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::DisableHelpSubcommand)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("verbose")
                .short('v')
                .multiple_occurrences(true)
                .global(true)
                .help("Increases logging verbosity each use for up to 2 times"),
        )
        .version(crate_version!())
}
