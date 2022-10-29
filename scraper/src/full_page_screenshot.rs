use anyhow::{Context, Result};
use fantoccini::{wd::WebDriverCompatibleCommand, Client};
use serde_json::{json, Value};

use rs_utils::decode_base64;

// TODO for Firefox / Geckodriver: let base64 = GET "/session/{sessionId}/moz/screenshot/full"

// https://chromedevtools.github.io/devtools-protocol/
#[derive(Debug)]
struct CDPCommand {
    command: String,
    params: Value,
}

impl CDPCommand {
    pub fn new(command: impl Into<String>, params: Value) -> Self {
        CDPCommand {
            command: command.into(),
            params,
        }
    }
}

// based on https://github.com/stevepryde/thirtyfour/blob/81bdd199e561f131dfab6d4a8a78a57cf7566917/thirtyfour/src/extensions/cdp/chromecommand.rs
impl WebDriverCompatibleCommand for CDPCommand {
    fn endpoint(
        &self,
        base_url: &url::Url,
        session_id: Option<&str>,
    ) -> Result<url::Url, url::ParseError> {
        let session_id = session_id.as_ref().expect("session_id must be present");

        base_url.join(&format!("session/{session_id}/goog/cdp/execute"))
    }

    fn method_and_body(&self, _request_url: &url::Url) -> (http::Method, Option<String>) {
        let body = json!({
            "cmd": self.command,
            "params": self.params,
        })
        .to_string();

        (http::Method::POST, Some(body))
    }
}

pub async fn cdp_capture_full_page_screenshot(client: &Client) -> Result<Vec<u8>> {
    let page_layout_metrics = client
        .issue_cmd(CDPCommand::new("Page.getLayoutMetrics", json!({})))
        .await
        .context("failed to run CDP command")?;

    let width = &page_layout_metrics["cssContentSize"]["width"];
    let height = &page_layout_metrics["cssContentSize"]["height"];

    let page_layout_metrics = client
        .issue_cmd(CDPCommand::new(
            "Page.captureScreenshot",
            json!({
                "captureBeyondViewport": true,
                "clip": {
                    "x": 0,
                    "y": 0,
                    "width": width,
                    "height": height,
                    "scale": 1,
                },
            }),
        ))
        .await
        .context("failed to run CDP command")?;

    let data = page_layout_metrics["data"]
        .as_str()
        .context("screenshot data is missing")?;

    decode_base64(data)
}
