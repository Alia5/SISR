use sdl3::event::Event;
use tracing::{debug, error, trace, warn};
use crate::app::steam_utils::binding_enforcer::binding_enforcer;

use crate::app::input::{device::SDLDevice, handler::ViiperEvent};

use super::EventHandler;

impl EventHandler {
    // Refresh all tracked gamepads when SDL signals UPDATE_COMPLETE (comes in as Unknown).
    pub fn on_update_complete(&mut self, which: u32) {
        let Ok(guard) = self.state.lock() else {
            error!(
                "Failed to acquire event handler state lock to handle update complete for SDL ID {}",
                which
            );
            return;
        };
        if guard.steam_overlay_open {
            trace!(
                "Skipping gamepad state update on SDL ID {} due to Steam overlay being open",
                which
            );
            return;
        }
        let Some((device_id, input_state)) = self.sdl_id_to_device.get_mut(&which) else {
            trace!("No tracked device for SDL ID {} on update complete", which);
            return;
        };
        if let Some(sdl_devices) = self.sdl_devices.get(&which)
            && let Some(gamepad) = sdl_devices.iter().find_map(|d| match d {
                SDLDevice::Gamepad(p) => Some(p),
                _ => None,
            })
        {
            input_state.update_from_sdl_gamepad(gamepad);
            self.viiper.update_device_state(device_id, input_state);
        } else {
            warn!("No tracked gamepad for SDL ID {} on update complete", which);
        }
    }
    pub fn on_pad_event(&mut self, event: &Event) {
        match event {
            Event::Unknown { .. } => {
                trace!("Unknown gamepad event: {:?}", event);
            }
            _ => {
                if event.is_joy() {
                    // Currently just drop lower level joystick events
                    return;
                }
                if !event.is_controller() {
                    warn!(
                        "Received non-gamepad/joystick event in on_pad_event: {:?}",
                        event
                    );
                    #[allow(clippy::single_match)]
                    return;
                }
                // handle all other events and just "update gamepad"
                // instead of duplicating code for every shit"
                // trace!("GamepadHandler: Pad event: {:?}", event);
            }
        }
    }

    pub fn on_viiper_event(&mut self, event: ViiperEvent) {
        match event {
            ViiperEvent::ServerDisconnected { device_id } => {
                // Needs to be done here to avoid deadlock
                self.viiper.remove_device(device_id);

                let Ok(mut guard) = self.state.lock() else {
                    error!("Failed to lock state for VIIPER disconnect handling");
                    return;
                };
                if let Some(device) = guard.devices.get_mut(&device_id) {
                    let had_steam_viiper = device.steam_handle != 0 && device.viiper_connected;

                    device.viiper_device = None;
                    device.viiper_connected = false;
                    debug!(
                        "Cleared VIIPER device for {} due to server disconnect",
                        device_id
                    );

                    if had_steam_viiper {
                        let has_any_steam_viiper = guard
                            .devices
                            .iter()
                            .any(|(_, d)| d.steam_handle != 0 && d.viiper_connected);
                        if !has_any_steam_viiper
                            && let Ok(mut enforcer) = binding_enforcer().lock()
                                && enforcer.is_active() {
                                    enforcer.deactivate();
                                }
                    }
                }
            }
            ViiperEvent::DeviceCreated {
                device_id,
                viiper_device,
            } => {
                let Ok(mut guard) = self.state.lock() else {
                    error!("Failed to lock state for VIIPER device created handling");
                    return;
                };
                let Some(device) = guard.devices.get_mut(&device_id) else {
                    warn!("Received created event for unknown device ID {}", device_id);
                    return;
                };
                device.viiper_device = Some(viiper_device);
                self.viiper.connect_device(device);
                self.request_redraw();
            }
            ViiperEvent::DeviceConnected { device_id } => {
                let Ok(mut guard) = self.state.lock() else {
                    error!("Failed to lock state for VIIPER device connected handling");
                    return;
                };
                let Some(device) = guard.devices.get_mut(&device_id) else {
                    warn!(
                        "Received connected event for unknown device ID {}",
                        device_id
                    );
                    return;
                };
                device.viiper_connected = true;

                if device.steam_handle != 0 {
                    let has_any_steam_viiper = guard
                        .devices
                        .iter()
                        .any(|(_, d)| d.steam_handle != 0 && d.viiper_connected);
                    if has_any_steam_viiper
                        && let Ok(mut enforcer) = binding_enforcer().lock()
                            && !enforcer.is_active() {
                                enforcer.activate();
                            }
                    }

                self.request_redraw();
            }
            ViiperEvent::DeviceRumble { device_id, l, r } => {
                warn!("Received rumble for device {}, l={}, r={}", device_id, l, r);

                let Ok(guard) = self.state.lock() else {
                    error!("Failed to lock state for rumble");
                    return;
                };

                let Some(device) = guard.devices.get(&device_id) else {
                    warn!("Device {} not found for rumble", device_id);
                    return;
                };

                let Some(&sdl_id) = device.sdl_ids.iter().next() else {
                    warn!("Device {} has no SDL IDs for rumble", device_id);
                    return;
                };
                drop(guard);

                let Some(devices) = self.sdl_devices.get_mut(&sdl_id) else {
                    warn!(
                        "No SDL devices found for SDL ID {} (device {}) in output event",
                        sdl_id, device_id
                    );
                    return;
                };

                let Some(gamepad) = devices.iter_mut().find_map(|d| match d {
                    SDLDevice::Gamepad(p) => Some(p),
                    _ => None,
                }) else {
                    warn!(
                        "No SDL gamepad found for SDL ID {} (device {}) in output event",
                        sdl_id, device_id
                    );
                    return;
                };
                if let Err(e) = gamepad.set_rumble(l as u16 * 257, r as u16 * 257, 10000) {
                    warn!("Failed to set rumble for device {}: {}", device_id, e);
                }
                self.request_redraw();
            }
            ViiperEvent::ErrorCreateDevice { device_id } => {
                error!("Failed to create VIIPER device {}", device_id);
                self.request_redraw();
            }
            ViiperEvent::ErrorConnectDevice { device_id } => {
                error!("Failed to connect VIIPER device {}", device_id);
                self.request_redraw();
            }
        }
    }
}
