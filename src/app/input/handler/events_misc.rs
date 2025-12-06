use tracing::{error, info, warn};

use super::EventHandler;

impl EventHandler {
    pub fn ignore_device(&mut self, device_id: u64) {
        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock to ignore device {}",
                device_id
            );
            return;
        };
        if let Some(device) = guard.devices.get(&device_id) {
            info!("Ignoring device {}: {}", device_id, device.id);
            guard.devices.remove(&device_id);
        } else {
            warn!("Tried to ignore unknown device ID {}", device_id);
        }
    }
}
