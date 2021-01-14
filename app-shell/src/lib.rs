use std::sync::Arc;

pub use builder::ActionHandler;
pub use builder::AppShellBuilder;
pub use context::AppShellContext;
pub use html_template::AppSource;
use rpc_message::{RpcMessage, RpcMessageResponse};
use warp::{reply, Reply};

mod builder;
mod context;
mod html_template;
mod rpc_message;
mod webview;

impl AppShellBuilder {
    pub async fn start(self, src: AppSource, handler: Arc<dyn ActionHandler>) {
        if cfg!(feature = "dev-server") {
            log::info!("Starting dev server");
            log::info!("source: {}", src);
            self.serve(src, handler).await;
        } else {
            self.load(src, handler);
        }
    }

    fn load(self, src: AppSource, handler: Arc<dyn ActionHandler>) {
        use gio::prelude::*;
        use gtk::prelude::*;
        use gtk::ApplicationWindow;
        use webkit2gtk::WebViewExt;
        use webview::build_webview;

        let application = gtk::Application::new(Some(&self.app_id), Default::default()).unwrap();

        let builder = Arc::new(self);
        application.connect_activate(move |app| {
            let (action_sender, action_receiver) =
                glib::MainContext::channel::<RpcMessage>(glib::PRIORITY_DEFAULT);

            let (action_response_sender, action_response_receiver) =
                glib::MainContext::channel::<RpcMessageResponse>(glib::PRIORITY_DEFAULT);

            {
                let builder = builder.clone();
                let handler = handler.clone();

                action_receiver.attach(None, move |msg| {
                    let builder = builder.clone();
                    let handler = handler.clone();
                    let action_response_sender = action_response_sender.clone();

                    glib::MainContext::default().spawn(async move {
                        let context = AppShellContext::new(builder.server_mode);
                        let result = handler.run(msg.action, &context, msg.params).await;

                        let response = match result {
                            Ok(result) => RpcMessageResponse {
                                call_id: msg.call_id,
                                result,
                                err: None,
                            },
                            Err(err) => RpcMessageResponse {
                                call_id: msg.call_id,
                                result: serde_json::Value::Null,
                                err: Some(err.to_string()),
                            },
                        };

                        action_response_sender
                            .send(response)
                            .expect("must be able to publish result");
                    });

                    glib::Continue(true)
                });
            }

            let webview = build_webview(
                builder.data_dir.clone(),
                action_sender,
                action_response_receiver,
            );

            webview.load_html(&src.render(&builder), Some(&src.get_base_path()));

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

    async fn serve(mut self, src: AppSource, handler: Arc<dyn ActionHandler>) {
        use rs_utils::TempFile;
        use std::fs;
        use std::process::Command;
        use tokio::signal;
        use warp::*;

        // for file picker
        gtk::init().expect("must be able to init gtk");

        self.server_mode = true;

        let src = Arc::new(src);
        let builder = Arc::new(self);

        // render temp file
        let temp_file = TempFile::new(&format!("{}.html", &builder.app_id));
        fs::write(temp_file.get_path(), AppSource::render(&src, &builder))
            .expect("failed to write data into temp file");
        println!("Root file: file:{}", temp_file.get_path());

        // create rpc handler
        let post_rpc = {
            let builder = builder.clone();
            let handler = handler.clone();

            let builder_filter = warp::any().map(move || builder.clone());
            let handler_filter = warp::any().map(move || handler.clone());

            let cors = warp::cors()
                .allow_any_origin()
                .allow_methods(&[http::Method::POST])
                .allow_header("content-type")
                .expose_header("content-type");

            warp::post()
                .and(warp::path("rpc"))
                .and(builder_filter.clone())
                .and(handler_filter.clone())
                .and(warp::body::json())
                .and_then(rpc_action_handler)
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
            .arg(format!("file:{}", temp_file.get_path()))
            .spawn()
            .expect("failed to run chromium");

        future.await.expect("failed to wait for server");
    }
}

async fn rpc_action_handler(
    builder: Arc<AppShellBuilder>,
    handler: Arc<dyn ActionHandler>,
    msg: RpcMessage,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::debug!("RPC MESSAGE: {}", msg);

    let context = AppShellContext::new(builder.server_mode);
    let result = handler.run(msg.action, &context, msg.params).await;

    let response = match result {
        Ok(result) => RpcMessageResponse {
            call_id: msg.call_id,
            result,
            err: None,
        },
        Err(err) => RpcMessageResponse {
            call_id: msg.call_id,
            result: serde_json::Value::Null,
            err: Some(err.to_string()),
        },
    };

    Ok(reply::json(&response).into_response())
}
