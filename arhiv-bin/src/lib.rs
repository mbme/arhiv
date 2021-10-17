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

use clap::{crate_version, App, AppSettings, Arg, SubCommand};

use arhiv_core::definitions::get_standard_schema;

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn build_app() -> App<'static, 'static> {
    App::new("arhiv")
        .bin_name("arhiv")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize Arhiv instance on local machine")
                .arg(
                    Arg::with_name("arhiv_id")
                        .long("arhiv_id")
                        .required(true)
                        .index(1)
                        .help("Arhiv id to use"),
                )
                .arg(
                    Arg::with_name("prime")
                        .long("prime")
                        .display_order(1)
                        .help("Initialize prime instance"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sync") //
                .about("Sync changes"),
        )
        .subcommand(
            SubCommand::with_name("apply-migrations")
                .about("Upgrade arhiv db schema to latest version"),
        )
        .subcommand(
            SubCommand::with_name("backup") //
                .about("Backup arhiv data"),
        )
        .subcommand(
            SubCommand::with_name("ui-server") //
                .about("Run arhiv UI server"),
        )
        .subcommand(
            SubCommand::with_name("ui-open") //
                .about("Open document in UI")
                .arg(
                    Arg::with_name("id")
                        .index(1)
                        .required(true)
                        .help("document id to open"),
                )
                .arg(
                    Arg::with_name("browser")
                        .long("browser")
                        .takes_value(true)
                        .min_values(0)
                        .env("BROWSER")
                        .help("Open using provided browser or fall back to $BROWSER env variable"),
                ),
        )
        .subcommand(
            SubCommand::with_name("prime-server")
                .about("Run prime server")
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .takes_value(true)
                        .default_value("23420")
                        .help("Listen on specific port"),
                ),
        )
        .subcommand(
            SubCommand::with_name("status") //
                .about("Print current status"),
        )
        .subcommand(
            SubCommand::with_name("config") //
                .about("Print config")
                .arg(
                    Arg::with_name("template")
                        .short("t")
                        .long("template")
                        .display_order(1)
                        .help("Prints config template"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get document by id")
                .arg(
                    Arg::with_name("id")
                        .required(true)
                        .help("id of the document"),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add new document")
                .arg(
                    Arg::with_name("document_type")
                        .required(true)
                        .possible_values(&get_standard_schema().get_document_types(true))
                        .index(1)
                        .help("One of known document types"),
                )
                .arg(
                    Arg::with_name("data")
                        .required(true)
                        .index(2)
                        .help("JSON object with document props"),
                ),
        )
        .subcommand(
            SubCommand::with_name("attach")
                .about("Add new attachment. Will hard link or copy file to arhiv.")
                .arg(
                    Arg::with_name("file_path")
                        .required(true)
                        .index(1)
                        .help("Absolute path to file to save"),
                )
                .arg(
                    Arg::with_name("move_file")
                        .short("m")
                        .help("Move file to arhiv"),
                ),
        )
        .subcommand(
            SubCommand::with_name("import")
                .about("Scrape data and create document")
                .arg(
                    Arg::with_name("url") //
                        .required(true)
                        .index(1)
                        .help("url to scrape"),
                )
                .arg(
                    Arg::with_name("skip_confirmation")
                        .long("skip_confirmation")
                        .help("Import scraped data without confirmation"),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .global(true)
                .help("Increases logging verbosity each use for up to 2 times"),
        )
        .version(crate_version!())
}
