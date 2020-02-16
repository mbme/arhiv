extern crate gio;
extern crate gtk;
extern crate webkit2gtk;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use webkit2gtk::{SettingsExt, WebView, WebViewExt};

fn main() {
    let application =
        Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default())
            .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("First GTK+ Program");
        window.set_default_size(350, 70);

        let html_content =
            "<html><body><h1>Hello, World!</h1><img src=\"./abstract/1.png\" /></body></html>";
        // https://docs.rs/webkit2gtk/0.8.0/webkit2gtk/trait.WebViewExt.html
        let webview = WebView::new();
        // webview.load_uri("https://crates.io/");
        webview.load_html(html_content, Some("file:///home/mbme/images/"));

        let settings = WebViewExt::get_settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);

        window.add(&webview);

        window.show_all();
    });

    application.run(&[]);
}
