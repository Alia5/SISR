use std::sync::{Arc, Mutex};

use crate::app::input::context::Context;
use crate::app::input::steamdeck_hid;

use viiper_client::devices::steamdeck;
use viiper_client::devices::steamdeck::SteamdeckOutput;

pub fn handle(ctx: Arc<Mutex<Context>>, device_id: &u64, output: &SteamdeckOutput) {
    tracing::trace!(
        "Handling VIIPER Steam Deck output for device ID {}",
        device_id
    );
    let Ok(ctx) = ctx.lock() else {
        tracing::error!("Failed to lock state for VIIPER device output handling");
        return;
    };
    let Some(device_mtx) = ctx.device_for_id(*device_id) else {
        tracing::warn!(
            "Received device output event for unknown device ID {}",
            device_id
        );
        return;
    };
    drop(ctx);
    let Ok(mut device) = device_mtx.lock() else {
        tracing::error!("Failed to lock device mutex for VIIPER device output handling");
        return;
    };

    let mut gp = None;
    for d in device.sdl_devices.iter_mut() {
        if d.gamepad.is_some() {
            gp = d.gamepad.as_mut();
            break;
        }
    }

    let Some(gamepad) = gp else {
        tracing::warn!("No gamepad found for device {}", device_id);
        return;
    };

    let Some(path) = gamepad.path() else {
        tracing::warn!("No path found for gamepad of device {}", device_id);
        return;
    };

    steamdeck_hid::send_whatever(&path, &output.payload);
}
