pub use builder::AppShellBuilder;
pub use context::AppShellContext;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
pub use html_template::*;
use std::rc::Rc;
use webkit2gtk::WebViewExt;
use webview::build_webview;

mod builder;
mod context;
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
                &src.render(&builder),
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
    pub async fn serve(mut self, src: AppSource) {
        use rpc_message::RpcMessage;
        use rs_utils::{is_production_mode, TempFile};
        use std::fs;
        use std::process::Command;
        use std::sync::Arc;
        use tokio::signal;
        use warp::*;

        if is_production_mode() {
            panic!("dev server must not be run in production mode")
        }

        // for file picker
        gtk::init().expect("must be able to init gtk");

        self.server_mode = true;

        let src = Arc::new(src);
        let builder = Arc::new(self);

        // render temp file
        let temp_file = TempFile::new(&format!("{}.html", &builder.app_id));
        fs::write(temp_file.get_path(), AppSource::render(&src, &builder))
            .expect("failed to write data into temp file");
        println!("Root file: file://{}", temp_file.get_path());

        // create rpc handler
        let post_rpc = {
            let builder = builder.clone();

            let cors = warp::cors()
                .allow_any_origin()
                .allow_methods(&[http::Method::POST])
                .allow_header("content-type")
                .expose_header("content-type");

            warp::post()
                .and(warp::path("rpc"))
                .and(warp::body::json())
                .map(move |msg: RpcMessage| {
                    let result = builder.handle_rpc_message(msg);

                    reply::json(&result).into_response()
                })
                .with(cors)
        };

        // run server
        let (addr, server) = warp::serve(post_rpc).bind_with_graceful_shutdown(
            ([127, 0, 0, 1], builder.server_port),
            async {
                signal::ctrl_c().await.expect("failed to listen for event");
                println!("\nGot Ctrl-C, stopping the server");
            },
        );
        let future = tokio::task::spawn(server);
        println!("RPC server listening on {}", addr);

        Command::new("chromium")
            .arg(format!("file://{}", temp_file.get_path()))
            .spawn()
            .expect("failed to run chromium");

        future.await.expect("failed to wait for server");
    }
}
