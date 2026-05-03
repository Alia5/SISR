use anyhow::{Context, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::app::{api, steam::cef_inject::util};

pub const DEFAULT_INJECT_TAB: &str = "SharedJSContext";
const DEFAULT_CEF_DEBUG_PORT: u16 = 8080;
const DEFAULT_INJECT_TIMEOUT: Duration = Duration::from_secs(5);

pub const CLEANUP_TABS: &[&str] = &[
    "SharedJSContext",
];

pub async fn inject<T>(js: &str) -> anyhow::Result<T>
where
	T: DeserializeOwned,
{
	inject_into_tab(DEFAULT_INJECT_TAB, js).await
}

pub async fn inject_into_tab<T>(tab: &str, js: &str) -> anyhow::Result<T>
where
	T: DeserializeOwned,
{
	let port = api::get_api_port()
		.ok_or_else(|| anyhow!("API port is not available yet"))?;
	let prefixed_js = format!("window.__SISR_API_URL = \"http://localhost:{port}\";\n{js}");

	let tabs = util::get_cef_tabs(DEFAULT_CEF_DEBUG_PORT)
		.await
		.context("failed to read Steam CEF tabs from remote debug endpoint")?;

	let websocket_url = tabs
		.into_iter()
		.find(|t| t.title.eq_ignore_ascii_case(tab))
		.map(|t| t.websocket_debugger_url)
		.ok_or_else(|| anyhow!("Steam CEF tab not found: {tab}"))?;

	timeout(
		DEFAULT_INJECT_TIMEOUT,
		execute_and_decode::<T>(&websocket_url, &prefixed_js),
	)
		.await
		.map_err(|_| anyhow!("timed out waiting for Steam CEF response"))?
}

async fn execute_and_decode<T>(websocket_url: &str, js: &str) -> anyhow::Result<T>
where
	T: DeserializeOwned,
{
	let (mut ws, _) = connect_async(websocket_url)
		.await
		.with_context(|| format!("failed to connect to Steam CEF websocket: {websocket_url}"))?;

	let request_id = 1_i64;
	let payload = json!({
		"id": request_id,
		"method": "Runtime.evaluate",
		"params": {
			"expression": js,
			"returnByValue": true,
			"awaitPromise": true
		}
	});

	ws.send(Message::Text(payload.to_string().into()))
		.await
		.context("failed to send Runtime.evaluate payload to Steam CEF")?;

	while let Some(message) = ws.next().await {
		let message = message.context("failed to read Steam CEF websocket response")?;

		let Message::Text(text) = message else {
			continue;
		};

		let raw: Value = serde_json::from_str(&text).context("failed to decode CEF response")?;

		if let Some(protocol_error) = raw.get("error") {
			return Err(anyhow!("CEF protocol response error: {protocol_error}"));
		}

		if raw.get("id").and_then(Value::as_i64) != Some(request_id) {
			continue;
		}

		let result_map = raw
			.get("result")
			.and_then(Value::as_object)
			.ok_or_else(|| anyhow!("malformed CEF response: missing result object"))?;

		if let Some(exception_details) = result_map.get("exceptionDetails") {
			return Err(anyhow!("javascript exception from CEF: {exception_details}"));
		}

		let runtime_result = result_map
			.get("result")
			.and_then(Value::as_object)
			.ok_or_else(|| anyhow!("malformed CEF response: missing runtime result"))?;

		let value = if let Some(value) = runtime_result.get("value") {
			value.clone()
		} else if runtime_result.get("type").and_then(Value::as_str) == Some("undefined") {
			Value::Null
		} else {
			return Err(anyhow!(
				"runtime result has no value and is not undefined"
			));
		};

		return serde_json::from_value(value).context("failed to decode CEF runtime value");
	}

	Err(anyhow!(
		"websocket closed before matching CEF response was received"
	))
}

