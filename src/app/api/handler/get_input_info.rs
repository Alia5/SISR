use std::collections::{BTreeMap, BTreeSet};

use axum::{Json, extract::State};
use reqwest::StatusCode;

use crate::{app::{api::AppState, input::device_info::SDLDeviceInfo, steam::binding_enforcer::binding_enforcer}, config::get_config};

/// Get Input Info
///
/// Returns information about the controllers detected
#[utoipa::path(
    get,
    path = "/api/v1/input_info",
    tag = "info",
    responses(
        (status = 200, body = InputInfoResponse),
    )
)]
pub async fn get_input_info(
    State(state): State<AppState>,
) -> (StatusCode, Json<InputInfoResponse>) {
    tracing::debug!("Received request for input info");

    let ctx = state.input_ctx.lock().expect("Failed to lock input context");

    let bus_ids: Vec<u32> = ctx
        .devices
        .iter()
        .filter_map(|entry| {
            let dev = entry.value().lock().ok()?;
            Some(dev.viiper_device.as_ref()?.device.bus_id)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let mut config_forced = false;
    let mut config_forced_app_id = None;
    if let Ok(enforcer) = binding_enforcer().lock() {
        config_forced = enforcer.is_active();
        config_forced_app_id = enforcer.app_id();
    }

    (StatusCode::OK, Json(InputInfoResponse {
        devices: ctx
            .devices
            .iter()
            .map(|entry| {
                let id = *entry.key();
                let device = entry.value().lock().expect("Failed to lock device");
                (id, DeviceInfo {
                    steam_handle: device.steam_handle,
                    viiper_type: device.viiper_type.clone(),
                    has_viiper_device: device.viiper_device.is_some(),
                    sdl_device_count: device.sdl_devices.len() as u32,
                    sdl_devices: device.sdl_devices.iter().map(|d| d.infos.clone()).collect(),
                    corresponding_device_id: device.corresponding_device_id,
                })
            })
            .collect(),
        viiper: ViiperInfo {
            address: ctx.viiper_address.map(|a| a.to_string()),
            address_is_loopback: ctx.viiper_address.map(|a| a.ip().is_loopback()).unwrap_or(false),
            available: ctx.viiper_available,
            version: ctx.viiper_version.clone(),
            bus_ids,
        },
        keyboard_mouse_emulation: ctx.keyboard_mouse_emulation,
        steam_overlay_open: ctx.steam_overlay_open,
        fullscreen: get_config().window.fullscreen.unwrap_or(true),
        force_controller_config: config_forced,
        force_controller_config_app_id: config_forced_app_id,
    }))
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct InputInfoResponse {
    pub devices: BTreeMap<u64, DeviceInfo>,
    pub viiper: ViiperInfo,
    pub keyboard_mouse_emulation: bool,
    pub steam_overlay_open: bool,
    pub fullscreen: bool,
    pub force_controller_config: bool,
    pub force_controller_config_app_id: Option<u32>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ViiperInfo {
    pub address: Option<String>,
    pub address_is_loopback: bool,
    pub available: bool,
    pub version: Option<String>,
    pub bus_ids: Vec<u32>,
}

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct DeviceInfo {
    pub steam_handle: u64,
    pub viiper_type: Option<String>,
    pub has_viiper_device: bool,
    pub sdl_device_count: u32,
    pub sdl_devices: Vec<SDLDeviceInfo>,
    pub corresponding_device_id: Option<u64>,
}