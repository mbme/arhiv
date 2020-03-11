use anyhow::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, ScrolledWindow};
use std::fs;
use std::path::Path;
use webkit2gtk::{SettingsExt, UserContentManagerExt, WebInspectorExt, WebView, WebViewExt};

pub struct AppShell {}

impl AppShell {
    pub fn show(app_id: &str, html_file: &Path) -> Result<AppShell> {
        if !gio::Application::id_is_valid(app_id) {
            return Err(anyhow!("Application id is invalid: {}", app_id));
        }

        let application = Application::new(Some(app_id), gio::ApplicationFlags::FLAGS_NONE)?;

        let webview = AppShell::load_file(html_file)?;

        application.connect_activate(move |app| {
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

        Ok(AppShell {})
    }

    fn load_file(html_file: &Path) -> Result<WebView> {
        let webview = WebView::new();
        let html_content = fs::read_to_string(html_file)?;
        webview.load_html(
            &html_content,
            Some(&format!("file://{}/", html_file.display())),
        );

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

        Ok(webview)
    }

    pub fn show_inspector(&self) {}
}
