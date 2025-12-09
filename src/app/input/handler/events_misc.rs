use tracing::{debug, error, info, warn};

use crate::{
    app::{
        gui::dialogs::{Dialog, push_dialog},
        steam_utils::{cef_debug, util::launched_via_steam},
    },
    config::CONFIG,
};

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

    pub fn connect_viiper_device(&mut self, device_id: u64) {
        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock to connect VIIPER device {}",
                device_id
            );
            return;
        };
        let Some(device) = guard.devices.get_mut(&device_id) else {
            warn!(
                "Tried to connect VIIPER for unknown device ID {}",
                device_id
            );
            return;
        };
        if device.viiper_connected || device.viiper_device.is_some() {
            warn!("Device ID {} is already connected to VIIPER", device_id);
            return;
        }
        if device.steam_handle == 0 {
            warn!(
                "Device ID {} does not have a valid Steam handle; cannot connect to VIIPER",
                device_id
            );
            return;
        }
        self.viiper.create_device(device);
        self.request_redraw();
    }

    pub fn disconnect_viiper_device(&mut self, device_id: u64) {
        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock to disconnect VIIPER device {}",
                device_id
            );
            return;
        };
        let Some(device) = guard.devices.get_mut(&device_id) else {
            warn!(
                "Tried to disconnect VIIPER for unknown device ID {}",
                device_id
            );
            return;
        };
        if !device.viiper_connected || device.viiper_device.is_none() {
            warn!("Device ID {} is not connected to VIIPER", device_id);
            return;
        }
        self.viiper.remove_device(device_id);
    }

    pub fn on_cef_debug_ready(&mut self, port: u16) {
        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock on CEF debug readiness on port {}",
                port
            );
            return;
        };
        guard.cef_debug_port = Some(port);
        self.request_redraw();
        // TODO: ready CEF debug when manually injecting overlay, patch this check, then
        let cont_redraw = guard.window_continuous_redraw.clone();
        if launched_via_steam() {
            if !cont_redraw.load(std::sync::atomic::Ordering::Relaxed) {
                self.async_handle.spawn(async move {
                    match cef_debug::inject::inject(
                        "Overlay",
                        str::from_utf8(cef_debug::payloads::OVERLAY_STATE_NOTIFIER)
                            .expect("Failed to convert overlay notifier payload to string"),
                    )
                    .await
                    {
                        Ok(_) => info!("Successfully injected CEF overlay state notifier"),
                        Err(e) => {
                            error!("Failed to inject CEF overlay state notifier: {}", e);
                            _ = push_dialog(Dialog::new_yes_no(
                                "Failed to init Steam overlay notifier",
                                format!(
                                    "SISR was not able to initialize the overlay notifier.
You may experience a non working Steam Overlay, unless you enable the \"Continous Redraw\" option.
This can cause higher CPU/GPU usage.
Enable continous redraw now?

\nError: {}",
                                    e
                                ),
                                move || {
                                    cont_redraw.store(true, std::sync::atomic::Ordering::Relaxed);
                                },
                                || {},
                            ));
                        }
                    }
                });
            } else {
                debug!(
                    "Skipping CEF overlay notifier injection due to continuous draw being enabled"
                );
            }
        } else {
            debug!("Not launched via Steam; skipping CEF overlay notifier injection");
        }
    }

    pub fn on_overlay_state_changed(&mut self, open: bool) {
        debug!("Steam overlay state changed event received: {}", open);
        if CONFIG.get().unwrap().window.continous_draw.unwrap() {
            debug!("Ignoring overlay state change due to continuous draw being enabled");
            return;
        }

        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock on Steam overlay state change to {}",
                open
            );
            return;
        };
        guard.steam_overlay_open = open;
        if !open {
            // wait a bit until disabling, to avoid steam overlay staying visible
            let cont_draw = guard.window_continuous_redraw.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                cont_draw.store(open, std::sync::atomic::Ordering::Relaxed);
            });
        } else {
            guard
                .window_continuous_redraw
                .store(open, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
