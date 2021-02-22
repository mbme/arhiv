#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use clap::{crate_version, App, Arg};
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;
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

    let application = Application::new(Some("v.binutils.filepicker"), Default::default())
        .expect("failed to initialize GTK application");

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

        let files = dialog.get_filenames();

        println!(
            "{}",
            serde_json::to_string(&files).expect("must be able to convert to string")
        );

        dialog.close();
    });

    application.run(&[]);
}
