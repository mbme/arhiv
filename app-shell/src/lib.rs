pub use crate::builder::AppShellBuilder;
use crate::webview::build_webview;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ScrolledWindow};
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

            let scrolled_window = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
            scrolled_window.set_min_content_width(builder.default_size.1);
            scrolled_window.add(webview.as_ref());

            let window = ApplicationWindow::new(app);
            window.set_title(&builder.title);
            window.set_default_size(builder.default_size.0, builder.default_size.1);
            window.add(&scrolled_window);

            window.show_all();

            if builder.show_inspector {
                let inspector = webview.get_inspector().unwrap();
                println!("attached: {}", inspector.is_attached());
                inspector.attach();
                println!("attached: {}", inspector.is_attached());
                inspector.show();
            }
        });

        application.run(&[]);
    }
}
