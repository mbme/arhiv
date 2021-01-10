use crate::context::AppShellContext;
use anyhow::*;
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::path::Path;

#[async_trait]
pub trait ActionHandler: Send + Sync {
    async fn run(&self, action: String, context: &AppShellContext, value: Value) -> Result<Value>;
}

pub struct AppShellBuilder {
    pub(crate) app_id: String,
    pub(crate) title: String,
    pub(crate) default_size: (i32, i32),
    pub(crate) data_dir: Option<String>,
    pub(crate) server_mode: bool,
    pub(crate) server_port: u16,
    pub(crate) js_variables: Map<String, Value>,
}

impl AppShellBuilder {
    pub fn create<S: Into<String>>(app_id: S) -> Self {
        let app_id = app_id.into();

        assert_eq!(gio::Application::id_is_valid(&app_id), true);

        AppShellBuilder {
            app_id,
            title: "".to_string(),
            default_size: (800, 600),
            data_dir: None,
            server_mode: false,
            server_port: 7001,
            js_variables: Map::new(),
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_default_size(mut self, width: i32, height: i32) -> Self {
        self.default_size = (width, height);
        self
    }

    pub fn with_data_dir<S: Into<String>>(mut self, data_dir: S) -> Self {
        let data_dir = data_dir.into();

        let path = Path::new(&data_dir);

        if !path.is_absolute() {
            panic!("data_dir must be absolute, got {}", data_dir);
        }

        self.data_dir = Some(data_dir);
        self
    }

    pub fn with_js_variable<S: Into<String>, V: Into<Value>>(
        mut self,
        variable: S,
        value: V,
    ) -> Self {
        self.js_variables.insert(variable.into(), value.into());
        self
    }

    // pub(crate) async fn handle_rpc_message(&self, message: RpcMessage) -> RpcMessageResponse {
    //     log::debug!("RPC MESSAGE: {}", message);

    //     self.action_channel
    //         .0
    //         .send(message)
    //         .expect("must be able to publish message");
    //     // let handler = match self.actions.get(&message.action) {
    //     //     Some(handler) => handler,
    //     //     None => {
    //     //         log::error!("RPC got unexpected action {}", message.action);

    //     //         return RpcMessageResponse {
    //     //             call_id: message.call_id,
    //     //             result: Value::Null,
    //     //             err: Some("Unknown action".to_string()),
    //     //         };
    //     //     }
    //     // };

    //     let context = AppShellContext::new(self.server_mode);

    //     match handler
    //         .handle(message.action, &context, message.params)
    //         .await
    //     {
    //         Ok(result) => RpcMessageResponse {
    //             call_id: message.call_id,
    //             result,
    //             err: None,
    //         },
    //         Err(err) => RpcMessageResponse {
    //             call_id: message.call_id,
    //             result: Value::Null,
    //             err: Some(err.to_string()),
    //         },
    //     }
    // }

    pub(crate) fn get_rpc_url(&self) -> String {
        format!("http://localhost:{}/rpc", self.server_port)
    }
}
