use crate::builder::AppShellBuilder;
use anyhow::*;
use glib::translate::from_glib_full;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use webkit2gtk::{
    LoadEvent, SettingsExt, UserContentManagerExt, WebContext, WebView, WebViewExt,
    WebsiteDataManager,
};
use webkit2gtk_sys::webkit_website_data_manager_new;

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

fn inject_rpc(webview: &WebView) {
    let ucm = webview.get_user_content_manager().unwrap();

    // register script message handler before injecting script so that window.webkit is immediately available
    ucm.register_script_message_handler("app-shell");

    webview.run_javascript(
        include_str!("./rpc.js"),
        None::<&gio::Cancellable>,
        |result| {
            if let Err(err) = result {
                log::error!("Failed to inject RPC script: {}", err);
                panic!("Failed to inject RPC script");
            }
        },
    );
}

// https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebsiteDataManager.html#webkit-website-data-manager-new
fn create_website_data_manager(data_dir: &str) -> WebsiteDataManager {
    unsafe {
        from_glib_full(webkit_website_data_manager_new(
            CString::new("base-cache-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("base-data-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("disk-cache-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("hsts-cache-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("indexeddb-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("local-storage-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("offline-application-cache-directory")
                .unwrap()
                .as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            CString::new("websql-directory").unwrap().as_ptr(),
            CString::new(data_dir).unwrap().as_ptr(),
            std::ptr::null::<i8>(),
        ))
    }
}

pub fn build_webview(builder: Rc<AppShellBuilder>, html_file: &Path) -> Rc<WebView> {
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

    let html_content = fs::read_to_string(html_file).unwrap();

    if !builder.actions.is_empty() {
        install_rpc_action_handler(&webview, builder);

        webview.connect_load_changed(move |webview, load_event| {
            log::debug!("webview load event {}", load_event);

            if load_event == LoadEvent::Committed {
                inject_rpc(webview);
            }
        });
    }

    webview.load_html(
        &html_content,
        Some(&format!("file://{}", html_file.display())),
    );

    // TODO render NOT FOUND if file is missing

    webview
}
