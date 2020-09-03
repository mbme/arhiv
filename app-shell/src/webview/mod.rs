use crate::builder::AppShellBuilder;
use crate::rpc_message::RpcMessage;
use std::rc::Rc;
use webkit2gtk::{
    SettingsExt, UserContentManagerExt, WebContext, WebView, WebViewExt, WebsiteDataManager,
};
use website_data_manager::create_website_data_manager;

mod website_data_manager;

fn install_rpc_action_handler(webview: &WebView, builder: Rc<AppShellBuilder>) {
    let ucm = webview.get_user_content_manager().unwrap();

    let webview = webview.clone();

    ucm.connect_script_message_received(move |_, result| {
        let msg: String = result
            .get_value()
            .unwrap()
            .to_string(&result.get_global_context().unwrap())
            .unwrap();

        let msg: RpcMessage = msg.parse().unwrap();

        let result = builder.handle_rpc_message(msg);

        webview.run_javascript(
            &format!("window.RPC._callResponse({});", result.serialize()),
            None::<&gio::Cancellable>,
            |result| {
                if let Err(err) = result {
                    log::error!("Failed to inject RPC response: {}", err);
                }
            },
        );
    });
}

pub fn build_webview(builder: Rc<AppShellBuilder>) -> Rc<WebView> {
    let data_manager = if let Some(ref path) = builder.data_dir {
        log::info!("website data manager: {}", path);
        create_website_data_manager(path)
    } else {
        log::info!("website data manager: ephemeral");
        WebsiteDataManager::new_ephemeral()
    };

    let web_context = WebContext::with_website_data_manager(&data_manager);
    let webview = Rc::new(WebView::with_context(&web_context));

    let settings = WebViewExt::get_settings(webview.as_ref()).unwrap();
    settings.set_enable_developer_extras(true);
    settings.set_allow_universal_access_from_file_urls(true);

    if !builder.actions.is_empty() {
        install_rpc_action_handler(&webview, builder);
        let ucm = webview.get_user_content_manager().unwrap();

        // register script message handler before injecting script so that window.webkit is immediately available
        ucm.register_script_message_handler("app-shell");
    }

    webview
}
