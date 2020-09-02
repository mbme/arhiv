use crate::builder::AppShellBuilder;
use rpc_message::RpcMessage;
use std::rc::Rc;
use webkit2gtk::{
    SettingsExt, UserContentManagerExt, WebContext, WebView, WebViewExt, WebsiteDataManager,
};
use website_data_manager::create_website_data_manager;

mod rpc_message;
mod website_data_manager;

fn install_rpc_action_handler(webview: &WebView, builder: Rc<AppShellBuilder>) {
    let ucm = webview.get_user_content_manager().unwrap();

    let webview = webview.clone();

    ucm.connect_script_message_received(move |_, result| {
        let rpc_message: String = result
            .get_value()
            .unwrap()
            .to_string(&result.get_global_context().unwrap())
            .unwrap();

        log::debug!("RPC MESSAGE: {}", rpc_message);

        let rpc_message: RpcMessage = rpc_message.parse().unwrap();

        if !builder.actions.contains_key(&rpc_message.action) {
            log::warn!("RPC got unexpected action {}", rpc_message.action);
            return;
        }

        let handler = builder.actions.get(&rpc_message.action).unwrap();

        let result = handler(rpc_message.params);

        webview.run_javascript(
            &format!(
                "window.RPC._callResult({}, {});",
                rpc_message.call_id, result
            ),
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
    settings.set_enable_developer_extras(builder.show_inspector);
    settings.set_allow_universal_access_from_file_urls(true);

    if !builder.actions.is_empty() {
        install_rpc_action_handler(&webview, builder);
        let ucm = webview.get_user_content_manager().unwrap();

        // register script message handler before injecting script so that window.webkit is immediately available
        ucm.register_script_message_handler("app-shell");
    }

    webview
}
