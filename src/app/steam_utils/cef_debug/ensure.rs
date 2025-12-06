use std::path::PathBuf;

use tracing::error;

use crate::app::steam_utils::util;

pub async fn check_enabled() -> bool {
    // http://localhost:8080/json/list <- tab list json / must contain "Steam" stuff.
    // TODO: Configurable port
    // NOTE: Steam itself does not provide a configurable port
    // TODO: create x-platform util to hook into steam to be able to change the port....
    // fuck me...

    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        reqwest::get("http://localhost:8080/json/list"),
    );

    if let Ok(Ok(response)) = timeout.await
        && let Ok(body) = response.text().await
    {
        return body.contains("Steam");
    }
    false
}

pub fn check_enable_file() -> bool {
    let Some(steam_path) = util::steam_path() else {
        error!("Steam path not found");
        return false;
    };

    let debug_file_path = steam_path.join(".cef-enable-remote-debugging");
    if debug_file_path.exists() {
        return true;
    }

    false
}
