use crate::builder::AppShellBuilder;
use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use webkit2gtk::{SettingsExt, UserContentManagerExt, WebView, WebViewExt};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RpcMessage {
    call_id: u32,
    action: String,
    params: Value,
}

impl std::str::FromStr for RpcMessage {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<RpcMessage> {
        serde_json::from_str(data).context("Failed to parse rpc message json")
    }
}

pub fn build_webview(builder: Rc<AppShellBuilder>, html_file: &Path) -> Rc<WebView> {
    let webview = Rc::new(WebView::new());

    let settings = WebViewExt::get_settings(webview.as_ref()).unwrap();
    settings.set_enable_developer_extras(builder.show_inspector);
    settings.set_allow_universal_access_from_file_urls(true);

    let html_content = fs::read_to_string(html_file).unwrap();

    webview.load_html(
        &html_content,
        Some(&format!("file://{}", html_file.display())),
    );

    // FIXME inject script also on reload
    if let Some(ref action_handler) = builder.action_handler {
        webview.run_javascript(include_str!("./rpc.js"), None::<&gio::Cancellable>, |_| {});

        let ucm = webview.get_user_content_manager().unwrap();
        {
            let result = ucm.register_script_message_handler("app-shell");
            assert_eq!(result, true);
        }

        let action_handler = action_handler.clone();
        let webview = webview.clone();
        ucm.connect_script_message_received(move |_, result| {
            let rpc_message: String = result
                .get_value()
                .unwrap()
                .to_string(&result.get_global_context().unwrap())
                .unwrap();

            log::debug!("RPC MESSAGE: {}", rpc_message);

            let rpc_message: RpcMessage = rpc_message.parse().unwrap();

            let result = action_handler(rpc_message.action, rpc_message.params);

            webview.run_javascript(
                &format!(
                    "window.RPC._callResult({}, {});",
                    rpc_message.call_id, result
                ),
                None::<&gio::Cancellable>,
                |_| {},
            );
        });
    }

    webview
}
