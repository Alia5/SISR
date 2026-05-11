use axum::{Json, extract::State};
use reqwest::StatusCode;
use serde_json::Value;

use crate::app::api::AppState;

fn default_config_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "SISR").map(|d| d.config_dir().join("SISR.json"))
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Default)]
#[serde(default)]
pub struct ConfigFileData {
    pub tray: Option<bool>,
    pub viiper_address: Option<String>,
    pub viiper_password: Option<String>,
    pub kbm_emulation: Option<bool>,
    pub update_notify: Option<String>,
    pub port: Option<u16>,
    pub window: Option<ConfigFileWindowOpts>,
    pub log: Option<ConfigFileLogOpts>,
    pub steam: Option<ConfigFileSteamOpts>,
    pub controller_emulation: Option<ConfigFileControllerEmulation>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Default)]
#[serde(default)]
pub struct ConfigFileWindowOpts {
    pub create: Option<bool>,
    pub fullscreen: Option<bool>,
    pub continuous_draw: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Default)]
#[serde(default)]
pub struct ConfigFileLogOpts {
    pub level: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Default)]
#[serde(default)]
pub struct ConfigFileSteamOpts {
    pub no_steam: Option<bool>,
    pub steam_path: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Default)]
#[serde(default)]
pub struct ConfigFileControllerEmulation {
    pub default_controller_type: Option<String>,
    pub require_controllers_connected_before_launch: Option<bool>,
    pub gyro_passthrough: Option<bool>,
    pub allow_desktop_config: Option<bool>,
}

/// Get Config File
///
/// Returns the contents of the config file at the default config path.
/// Fields not present in the file are omitted.
#[utoipa::path(
    get,
    path = "/api/v1/config",
    tag = "config",
    responses(
        (status = 200, body = ConfigFileData),
        (status = 404, description = "No config file found at default path"),
    )
)]
pub async fn get_config_file(
    State(_state): State<AppState>,
) -> Result<Json<ConfigFileData>, StatusCode> {
    let path = default_config_path().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    if !path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
    let raw = std::fs::read_to_string(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let parsed: ConfigFileData =
        serde_json::from_str(&raw).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(parsed))
}

/// Update Config File
///
/// Merges the provided fields into the config file at the default config path.
/// Only fields present in the request body are written; all fields are optional.
#[utoipa::path(
    patch,
    path = "/api/v1/config",
    tag = "config",
    request_body = ConfigFileData,
    responses(
        (status = 200, description = "OK"),
    )
)]
pub async fn update_config_file(
    State(_state): State<AppState>,
    Json(patch): Json<ConfigFileData>,
) -> StatusCode {
    let path = match default_config_path() {
        Some(p) => p,
        None => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let mut current: Value = if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or(Value::Object(Default::default())),
            Err(_) => Value::Object(Default::default()),
        }
    } else {
        Value::Object(Default::default())
    };

    let patch_value = match serde_json::to_value(patch) {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    merge_json(&mut current, patch_value);

    let json_str = match serde_json::to_string_pretty(&current) {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    match std::fs::write(&path, json_str) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn merge_json(base: &mut Value, patch: Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (k, v) in patch_map {
                if v.is_null() {
                    continue;
                }
                merge_json(base_map.entry(k).or_insert(Value::Null), v);
            }
        }
        (base, patch) => *base = patch,
    }
}
