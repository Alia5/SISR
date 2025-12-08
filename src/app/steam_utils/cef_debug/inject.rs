use std::sync::OnceLock;

use crate::app::steam_utils::cef_debug::ensure::CEF_DEBUG_PORT;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use tracing::trace;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CefTab {
    pub description: String,
    pub devtools_frontend_url: String,
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub tab_type: String,
    pub url: String,
    pub web_socket_debugger_url: String,
}

static WS_SERVER_PORT: OnceLock<u16> = OnceLock::new();

pub fn set_ws_server_port(port: u16) -> bool {
    WS_SERVER_PORT.set(port).is_ok()
}

pub fn get_ws_server_port() -> Option<u16> {
    WS_SERVER_PORT.get().cloned()
}

pub fn sisr_host() -> String {
    match get_ws_server_port() {
        Some(port) => format!("localhost:{}", port),
        None => "localhost:0".to_string(),
    }
}

pub async fn inject(tab_title: &str, payload: &str) -> Result<String> {
    let ws_port = WS_SERVER_PORT
        .get()
        .cloned()
        .expect("WebSocket server port not set");

    let mut tab_title = tab_title.to_string();
    let tabs = list_tabs().await?;
    // TODO: needs better handling, but for now, inject into the first overlay-tab we can find
    // as it is likely the most recent anyway
    if tab_title == "Overlay" {
        tab_title = tabs
            .iter()
            .find(|t| t.title.contains("Overlay"))
            .map(|t| t.title.clone())
            .ok_or_else(|| anyhow::anyhow!("Overlay tab not found"))?;
    }
    let tab = tabs
        .iter()
        .find(|t| t.title == tab_title)
        .ok_or_else(|| anyhow::anyhow!("Tab with title '{}' not found", tab_title))?;

    let ws_url = tab.web_socket_debugger_url.clone();
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let js_payload = format!("var SISR_HOST = 'localhost:{}';\n{}", ws_port, payload);

    let command = serde_json::json!({
        "id": 1,
        "method": "Runtime.evaluate",
        "params": {
            "expression": js_payload,
            "returnByValue": true
        }
    });

    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    write
        .send(Message::Text(command.to_string().into()))
        .await?;

    if let Some(msg) = read.next().await {
        let response = msg?;
        if let Message::Text(text) = response {
            let text_str = text.to_string();
            let result: serde_json::Value = serde_json::from_str(&text_str)?;

            if let Some(exception_details) =
                result.get("result").and_then(|r| r.get("exceptionDetails"))
            {
                return Err(anyhow::anyhow!(
                    "JavaScript exception: {}",
                    serde_json::to_string_pretty(&exception_details)?
                ));
            }

            if let Some(exception_details) = result.get("exceptionDetails") {
                return Err(anyhow::anyhow!(
                    "JavaScript exception: {}",
                    serde_json::to_string_pretty(&exception_details)?
                ));
            }

            if let Some(error) = result.get("error") {
                return Err(anyhow::anyhow!(
                    "Chrome DevTools Protocol error: {}",
                    serde_json::to_string_pretty(&error)?
                ));
            }

            if let Some(result_value) = result.get("result").and_then(|r| r.get("result"))
                && let Some(value) = result_value.get("value")
            {
                trace!("JavaScript injection result: {}", value);
                return Ok(value.to_string());
            }

            Ok("undefined".to_string())
        } else {
            Err(anyhow::anyhow!("Unexpected WebSocket message type"))
        }
    } else {
        Err(anyhow::anyhow!("No response from WebSocket"))
    }
}

pub async fn list_tabs() -> Result<Vec<CefTab>> {
    let cef_port = CEF_DEBUG_PORT
        .get()
        .cloned()
        .expect("CEF debug port not set");
    let url = format!("http://localhost:{}/json/list", cef_port);
    let resp = reqwest::get(&url).await?.text().await?;
    let tabs: Vec<CefTab> = serde_json::from_str(&resp)?;
    Ok(tabs)
}
