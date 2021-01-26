use crate::rpc_message::RpcMessage;
use crate::rpc_message::RpcMessageResponse;
use std::rc::Rc;
use tracing::{error, info};
use webkit2gtk::{
    SettingsExt, UserContentManagerExt, WebContext, WebView, WebViewExt, WebsiteDataManager,
};
use website_data_manager::create_website_data_manager;

mod website_data_manager;

pub fn build_webview(
    data_dir: Option<String>,
    actions: glib::Sender<RpcMessage>,
    responses: glib::Receiver<RpcMessageResponse>,
) -> Rc<WebView> {
    let data_manager = if let Some(ref path) = data_dir {
        info!("website data manager: {}", path);
        create_website_data_manager(path)
    } else {
        info!("website data manager: ephemeral");
        WebsiteDataManager::new_ephemeral()
    };

    let web_context = WebContext::with_website_data_manager(&data_manager);
    let webview = Rc::new(WebView::with_context(&web_context));

    let settings = WebViewExt::get_settings(webview.as_ref()).unwrap();
    settings.set_enable_developer_extras(true);
    settings.set_allow_universal_access_from_file_urls(true);

    let ucm = webview.get_user_content_manager().unwrap();

    // listen for actions
    {
        ucm.connect_script_message_received(move |_, result| {
            let msg: String = result
                .get_value()
                .unwrap()
                .to_string(&result.get_global_context().unwrap())
                .unwrap();

            let msg: RpcMessage = msg.parse().expect("must parse rpc message");

            actions.send(msg).expect("must publish rpc message");
        });
    }

    // send action responses
    {
        let webview = webview.clone();

        responses.attach(None, move |result| {
            webview.run_javascript(
                &format!("window.RPC._callResponse({});", result.serialize()),
                None::<&gio::Cancellable>,
                |result| {
                    if let Err(err) = result {
                        error!("Failed to inject RPC response: {}", err);
                    }
                },
            );
            glib::Continue(true)
        });
    }

    // register script message handler before injecting script so that window.webkit is immediately available
    ucm.register_script_message_handler("app-shell");

    webview
}
