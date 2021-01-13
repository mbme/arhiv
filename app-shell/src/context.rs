use gtk::*;
use std::path::PathBuf;

pub struct AppShellContext {
    server_mode: bool,
}

impl AppShellContext {
    pub(crate) fn new(server_mode: bool) -> Self {
        AppShellContext { server_mode }
    }

    pub fn pick_files(&self, select_multiple: bool) -> Vec<PathBuf> {
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

        if self.server_mode {
            while gtk::events_pending() {
                gtk::main_iteration();
            }
        }

        files
    }

    pub fn copy_to_clipboard(&self, data: &str) {
        let display = gdk::Display::get_default().unwrap();

        gtk::Clipboard::get_default(&display)
            .unwrap()
            .set_text(data);
    }
}
