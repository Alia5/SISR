use std::sync::{Arc, Mutex};

use viiper_client::devices::dualshock4::Dualshock4Output;

use crate::app::input::context::Context;

pub fn handle_output(ctx: Arc<Mutex<Context>>, device_id: &u64, output: &Dualshock4Output) {
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
            if let Err(e) = gamepad.set_rumble(
                output.rumble_large as u16 * 257,
                output.rumble_small as u16 * 257,
                10000,
            ) {
                tracing::warn!("Failed to set rumble for device {}: {}", device_id, e);
            }

            if unsafe { gamepad.has_led() } {
                // MIMIMI clippy can be collapsed mimimi
                if gamepad
                    .set_led(output.led_red, output.led_green, output.led_blue)
                    .is_err()
                {
                    tracing::warn!("Failed to set LED for device {}", device_id);
                }
            }
        } else {
            tracing::error!("No gamepad found for device {}", device_id);
        }
    }
}
