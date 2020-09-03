pub use crate::builder::AppShellBuilder;
pub use crate::file_picker::pick_files;
use crate::webview::build_webview;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
pub use html_template::*;
use std::rc::Rc;
use webkit2gtk::WebViewExt;

mod builder;
mod file_picker;
mod html_template;
mod rpc_message;
mod webview;

impl AppShellBuilder {
    pub fn load(self, src: AppSource) {
        let application =
            Application::new(Some(&self.app_id), gio::ApplicationFlags::FLAGS_NONE).unwrap();

        let builder = Rc::new(self);
        application.connect_activate(move |app| {
            let webview = build_webview(builder.clone());

            webview.load_html(
                &src.render(&builder.title, false),
                src.get_base_path()
                    .map(|path| format!("file://{}", path))
                    .as_deref(),
            );

            let window = ApplicationWindow::new(app);
            window.set_title(&builder.title);
            window.set_default_size(builder.default_size.0, builder.default_size.1);
            window.add(webview.as_ref());

            // reload on F5 and Ctrl-r
            {
                let webview = webview.clone();
                window.connect_key_press_event(move |_, key| {
                    if key.get_keyval() == gdk::keys::constants::F5 {
                        webview.reload();
                    }

                    if key.get_state() == gdk::ModifierType::CONTROL_MASK
                        && key.get_keyval() == gdk::keys::constants::r
                    {
                        webview.reload();
                    }

                    Inhibit(false)
                });
            }

            window.show_all();
        });

        application.run(&[]);
    }

    #[tokio::main]
    pub async fn serve(self, src: AppSource) {
        use rpc_message::RpcMessage;
        use std::sync::Arc;
        use warp::{reply, Filter, Reply};

        // for file picker
        gtk::init().expect("must be able to init gtk");

        let src = Arc::new(src);
        let builder = Arc::new(self);

        let post_rpc = {
            let builder = builder.clone();

            warp::post()
                .and(warp::path("rpc"))
                .and(warp::body::json())
                .map(move |msg: RpcMessage| {
                    let result = builder.handle_rpc_message(msg);

                    reply::json(&result).into_response()
                })
        };

        let get_app = {
            let src = src.clone();
            let builder = builder.clone();

            warp::get()
                .and(warp::path::end())
                .map(move || reply::html(AppSource::render(&src, &builder.title, true)))
        };

        let get_favicon = warp::get()
            .and(warp::path("favicon.ico"))
            .map(|| reply::with_status("", warp::http::StatusCode::NOT_FOUND));

        let routes = get_app.or(post_rpc).or(get_favicon);

        // TODO serve dir if AppSource is File

        warp::serve(routes).run(([127, 0, 0, 1], 7001)).await;
    }
}
