use tracing::{debug, error, info, trace, warn};

use std::sync::atomic::Ordering;

use crate::{
    app::{
        gui::dialogs::{self, Dialog, push_dialog},
        steam_utils::{cef_debug, util::launched_via_steam},
    },
    config::CONFIG,
};

use super::EventHandler;

impl EventHandler {
    pub fn set_kbm_emulation_enabled(&mut self, enabled: bool) {
        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock to set kbm emulation enabled={}",
                enabled
            );
            return;
        };

        if guard.kbm_emulation_enabled == enabled {
            return;
        }

        guard.kbm_emulation_enabled = enabled;
        self.kbm_emulation_enabled.store(enabled, Ordering::Relaxed);
        info!("KB/M emulation toggled: {}", enabled);

        if let Ok(guard_proxy) = self.winit_waker.lock()
            && let Some(proxy) = guard_proxy.as_ref()
            && let Err(e) =
                proxy.send_event(crate::app::window::RunnerEvent::SetKbmCursorGrab(enabled))
        {
            warn!("Failed to notify window about KB/M cursor grab toggle: {e}");
        }

        // When enabling, show a single OK dialog. On OK we enter capture mode.
        if enabled {
            const TITLE: &str = "KB/M emulation";
            let already_open = dialogs::REGISTRY
                .get()
                .map(|r| r.snapshot_dialogs().iter().any(|d| d.title == TITLE))
                .unwrap_or(false);

            if !already_open {
                let winit_waker = self.winit_waker.clone();
                let msg = "KB/M emulation enabled.\n\n\
UI will be hidden and the cursor will be captured when you enter capture mode.\n\n\
Toggle UI/capture:\n\
  Keyboard: Ctrl+Shift+Alt+S\n\
  Gamepad:  LB+RB+Back+A";
                _ = push_dialog(Dialog::new_ok(TITLE, msg, move || {
                    if let Ok(guard) = winit_waker.lock()
                        && let Some(proxy) = guard.as_ref()
                        && let Err(e) =
                            proxy.send_event(crate::app::window::RunnerEvent::EnterCaptureMode())
                    {
                        warn!("Failed to enter capture mode after KB/M OK: {e}");
                    }
                }))
                .inspect_err(|e| warn!("Failed to push KB/M emulation dialog: {e}"));
            }
        }

        guard.kbm_keyboard_modifiers = 0;
        guard.kbm_keyboard_keys.clear();
        guard.kbm_mouse_buttons = 0;

        if enabled {
            let has_keyboard = guard.devices.values().any(|d| d.viiper_type == "keyboard");
            let has_mouse = guard.devices.values().any(|d| d.viiper_type == "mouse");

            if !has_keyboard {
                let keyboard_id = self.next_device_id;
                self.next_device_id += 1;

                let keyboard_device = crate::app::input::device::Device {
                    id: keyboard_id,
                    viiper_type: "keyboard".to_string(),
                    ..Default::default()
                };
                if guard.viiper_ready {
                    self.viiper.create_device(&keyboard_device);
                } else {
                    info!(
                        "VIIPER not ready; scheduling keyboard device {} for connect on ready",
                        keyboard_id
                    );
                }
                guard.devices.insert(keyboard_id, keyboard_device);
            }

            if !has_mouse {
                let mouse_id = self.next_device_id;
                self.next_device_id += 1;

                let mouse_device = crate::app::input::device::Device {
                    id: mouse_id,
                    viiper_type: "mouse".to_string(),
                    ..Default::default()
                };
                if guard.viiper_ready {
                    self.viiper.create_device(&mouse_device);
                } else {
                    info!(
                        "VIIPER not ready; scheduling mouse device {} for connect on ready",
                        mouse_id
                    );
                }
                guard.devices.insert(mouse_id, mouse_device);
            }
        } else {
            let ids: Vec<u64> = guard
                .devices
                .iter()
                .filter_map(|(id, d)| {
                    if d.viiper_type == "keyboard" || d.viiper_type == "mouse" {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();

            for id in ids {
                if guard.viiper_ready {
                    self.viiper.remove_device(id);
                } else {
                    trace!(
                        "VIIPER not ready; skip removing KB/M device {} from VIIPER",
                        id
                    );
                }
                guard.devices.remove(&id);
            }
        }
    }

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
        let viiper_ready = guard.viiper_ready;
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
        if viiper_ready {
            self.viiper.create_device(device);
        } else {
            info!(
                "VIIPER not ready; will connect device {} when VIIPER is ready",
                device_id
            );
        }
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
        let viiper_ready = guard.viiper_ready;
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
        if viiper_ready {
            self.viiper.remove_device(device_id);
        } else {
            trace!(
                "VIIPER not ready; skipping disconnect for device {} (no server connection yet)",
                device_id
            );
        }
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

        let cont_redraw = guard.window_continuous_redraw.clone();
        if !cont_redraw.load(std::sync::atomic::Ordering::Relaxed) {
            self.async_handle.spawn(async move {
                if !launched_via_steam() {
                    debug!("NOT launched via Steam, delaying CEF overlay notifier injection");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    // TODO: FIXME!
                }
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
            debug!("Skipping CEF overlay notifier injection due to continuous draw being enabled");
        }
    }

    pub fn on_overlay_state_changed(&mut self, open: bool) {
        debug!("Steam overlay state changed event received: {}", open);
        let continous_draw_in_config = CONFIG
            .read()
            .ok()
            .and_then(|c| {
                c.as_ref()
                    .map(|cfg| cfg.window.continous_draw.unwrap_or(false))
            })
            .unwrap_or(false);
        if continous_draw_in_config {
            debug!("Ignoring overlay state change due to continuous draw being enabled in config");
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
        drop(guard);

        let Ok(waker_guard) = self.winit_waker.lock() else {
            error!(
                "Failed to acquire winit waker lock on Steam overlay state change to {}",
                open
            );
            return;
        };
        if let Some(proxy) = waker_guard.as_ref()
            && let Err(e) =
                proxy.send_event(crate::app::window::RunnerEvent::OverlayStateChanged(open))
        {
            error!("Failed to send overlay state change to window: {}", e);
        }
    }

    pub fn on_viiper_ready(&mut self, version: String) {
        info!("VIIPER reported ready with version {}", version);
        let Ok(mut guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock on VIIPER ready with version {}",
                version
            );
            return;
        };
        guard.viiper_version = Some(version.clone());
        guard.viiper_ready = true;
        self.request_redraw();
        for device in guard.devices.values_mut() {
            if device.viiper_device.is_none() {
                if device.viiper_type == "keyboard" {
                    info!(
                        "connecting keyboard device ID {} to VIIPER now that VIIPER is ready",
                        device.id
                    );
                    self.viiper.create_device(device);
                    continue;
                }
                if device.viiper_type == "mouse" {
                    info!(
                        "connecting mouse device ID {} to VIIPER now that VIIPER is ready",
                        device.id
                    );
                    self.viiper.create_device(device);
                    continue;
                }
                if self.steamdeck_gamepad_direct_forward
                    && device.viiper_type == "steamdeck"
                    && device.steam_handle == 0
                {
                    info!(
                        "connecting steamdeck device ID {} to VIIPER now that VIIPER is ready",
                        device.id
                    );
                    self.viiper.create_device(device);
                    continue;
                }
                if device.steam_handle != 0 {
                    info!(
                        "Connecting device ID {} to VIIPER now that VIIPER is ready",
                        device.id
                    );
                    self.viiper.create_device(device);
                } else {
                    info!(
                        "Skipping device ID {} VIIPER connection; no valid Steam handle",
                        device.id
                    );
                }
            }
        }
    }
}
