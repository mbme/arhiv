use gtk::*;
use std::path::PathBuf;

pub fn pick_files(select_multiple: bool) -> Vec<PathBuf> {
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

    dialog.close();

    files
}
