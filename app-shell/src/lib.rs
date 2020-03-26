pub use crate::builder::AppShellBuilder;
use crate::webview::build_webview;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use std::path::Path;
use std::rc::Rc;
use webkit2gtk::{WebInspectorExt, WebViewExt};

mod builder;
mod webview;

impl AppShellBuilder {
    pub fn load(self, html_file: String) {
        let application =
            Application::new(Some(&self.app_id), gio::ApplicationFlags::FLAGS_NONE).unwrap();

        let builder = Rc::new(self);
        application.connect_activate(move |app| {
            let html_file = Path::new(&html_file);
            let webview = build_webview(builder.clone(), &html_file);

            let window = ApplicationWindow::new(app);
            window.set_title(&builder.title);
            window.set_default_size(builder.default_size.0, builder.default_size.1);
            window.add(webview.as_ref());

            // reload on F5 and Ctrl-r
            {
                let webview = webview.clone();
                window.connect_key_press_event(move |_, key| {
                    if key.get_keyval() == gdk::enums::key::F5 {
                        webview.reload();
                    }

                    if key.get_state() == gdk::ModifierType::CONTROL_MASK
                        && key.get_keyval() == gdk::enums::key::r
                    {
                        webview.reload();
                    }

                    Inhibit(false)
                });
            }

            window.show_all();

            if builder.show_inspector {
                let inspector = webview.get_inspector().unwrap();
                inspector.show();
            }
        });

        application.run(&[]);
    }
}
