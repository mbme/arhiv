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

use clap::{crate_version, App, Arg};
use gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{Application, FileChooserAction, FileChooserDialog, ResponseType, Window};

use rs_utils::log::setup_logger;

fn main() {
    setup_logger();

    let app = App::new("mb-filepicker")
        .arg(
            Arg::with_name("multiple")
                .short("m")
                .help("Allow to pick multiple files"),
        )
        .version(crate_version!());

    let matches = app.get_matches();

    let select_multiple = matches.is_present("multiple");

    let application = Application::new(Some("v.binutils.filepicker"), ApplicationFlags::default());

    application.connect_activate(move |_app| {
        let title = if select_multiple {
            "Open Files"
        } else {
            "Open File"
        };

        let dialog = FileChooserDialog::with_buttons::<Window>(
            Some(title),
            None,
            FileChooserAction::Open,
            &[
                ("_Cancel", ResponseType::Cancel),
                ("_Open", ResponseType::Accept),
            ],
        );
        dialog.set_select_multiple(select_multiple);

        dialog.run();

        let files = dialog.filenames();

        println!(
            "{}",
            serde_json::to_string(&files).expect("must be able to convert to string")
        );

        dialog.close();
    });

    application.run();
}
