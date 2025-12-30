use std::sync::{Arc, Mutex};

use crate::app::input::context::Context;

pub fn handle(ctx: Arc<Mutex<Context>>, device_id: &u64, rumble_left: &u8, rumble_right: &u8) {
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

    for d in device.sdl_devices.iter_mut() {
        if let Some(gamepad) = d.gamepad.as_mut() {
            // FUCK CLIPPY!
            if let Err(e) =
                gamepad.set_rumble(*rumble_left as u16 * 257, *rumble_right as u16 * 257, 10000)
            {
                tracing::warn!("Failed to set rumble for device {}: {}", device_id, e);
            }
        } else {
            tracing::error!("No gamepad found for device {}", device_id);
        }
    }
}
