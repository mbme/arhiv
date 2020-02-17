extern crate gio;
extern crate gtk;
extern crate webkit2gtk;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ScrolledWindow};
use std::fs;
use webkit2gtk::{SettingsExt, UserContentManagerExt, WebInspectorExt, WebView, WebViewExt};

fn main() {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let current_dir = format!("{}/src", manifest_dir);
    let html_content = fs::read_to_string(format!("{}/index.html", current_dir)).unwrap();

    application.connect_activate(move |app| {
        // https://docs.rs/webkit2gtk/0.8.0/webkit2gtk/trait.WebViewExt.html
        let webview = WebView::new();
        // webview.load_uri("https://crates.io/");
        webview.load_html(&html_content, Some(&format!("file://{}/", current_dir)));

        let ucm = webview.get_user_content_manager().unwrap();
        {
            let result = UserContentManagerExt::register_script_message_handler(&ucm, "test");
            println!("registered {}", result);
        }
        UserContentManagerExt::connect_script_message_received(&ucm, |_, result| {
            println!("got {:#?}", result);
        });

        // webview.connect_run_file_chooser(|_webview, _fc| -> bool {
        //     let fc = gtk::FileChooserDialogBuilder::new().build();
        //     fc.run();
        //     true
        // });

        let settings = WebViewExt::get_settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);
        settings.set_allow_universal_access_from_file_urls(true);

        let scrolled_window = ScrolledWindow::new(gtk::NONE_ADJUSTMENT, gtk::NONE_ADJUSTMENT);
        scrolled_window.set_min_content_width(600);
        scrolled_window.add(&webview);

        let window = ApplicationWindow::new(app);
        window.set_title("First GTK+ Program");
        window.set_default_size(350, 70);
        window.add(&scrolled_window);

        window.show_all();

        let inspector = webview.get_inspector().unwrap();
        println!("attached: {}", inspector.is_attached());
        inspector.attach();
        println!("attached: {}", inspector.is_attached());
        inspector.show();

        webview.run_javascript(
            "window._MBME = 'test';",
            None::<&gio::Cancellable>,
            |result| {
                println!("execution result {:?}", result);
            },
        );

        // window.connect_delete_event(|_, _| {
        //     gtk::main_quit();
        //     Inhibit(false)
        // });
    });

    application.run(&[]);
}
