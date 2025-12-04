use sdl3::event::Event;
use tracing::{debug, error, info, trace, warn};

use crate::{
    app::input::{
        device::{Device, DeviceState, SDLDevice},
        handler::ViiperEvent,
        sdl::get_gamepad_steam_handle,
        sdl_device_info::SdlDeviceInfo,
    },
    event_which,
};

use super::EventHandler;

impl EventHandler {
    pub fn on_pad_event(&self, event: &Event) {
        match event {
            Event::Unknown { .. } => {
                // if nothing "outside" changed is
                // GAMEPAD_STATE_UPDATE_COMPLETE or JOYPAD_STATE_UPDATE_COMPLETE
                // Silently Ignore for now
                // Would need "supertrace" log level lol
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
                    return;
                }
                // handle all other events and just "update gamepad"
                // instead of duplicating code for every shit"
                trace!("GamepadHandler: Pad event: {:?}", event);
                let Some(which) = event_which!(event) else {
                    warn!("Failed to get 'which' from gamepad event: {:?}", event);
                    return;
                };

                let Some(&device_id) = self.sdl_id_to_device.get(&which) else {
                    warn!("No device found for SDL ID {} in pad event", which);
                    return;
                };

                if let Ok(mut guard) = self
                    .state
                    .lock()
                    .map_err(|e| error!("Failed to lock state for pad event: {}", e))
                    && let Some(device) = guard.devices.iter_mut().find(|d| d.id == device_id)
                {
                    let Some(gamepad) = self.sdl_devices.get(&which).and_then(|devs| {
                        devs.iter()
                            .find(|d| matches!(d, SDLDevice::Gamepad(_)))
                            .and_then(|d| match d {
                                SDLDevice::Gamepad(p) => Some(p),
                                _ => None,
                            })
                    }) else {
                        warn!("No SDL gamepad found for SDL ID {}", which);
                        return;
                    };

                    if device.steam_handle == 0 {
                        let handle = get_gamepad_steam_handle(gamepad);
                        if handle != 0 {
                            device.steam_handle = handle;
                            self.viiper.create_device(device);
                            return;
                        }
                        warn!(
                            "Device {} (SDL ID {}) has no steam handle in pad event",
                            device_id, which
                        );
                        return;
                    }

                    device.state.update_from_sdl_gamepad(gamepad);

                    self.viiper.update_device_state(device);
                } else {
                    warn!("Device {} not found in state for pad event", device_id);
                }
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
                if let Some(device) = guard.devices.iter_mut().find(|d| d.id == device_id) {
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
                            .any(|d| d.steam_handle != 0 && d.viiper_connected);
                        if !has_any_steam_viiper && guard.binding_enforcer.is_active() {
                            guard.binding_enforcer.deactivate();
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
                let Some(device) = guard.devices.iter_mut().find(|d| d.id == device_id) else {
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
                let Some(device) = guard.devices.iter_mut().find(|d| d.id == device_id) else {
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
                        .any(|d| d.steam_handle != 0 && d.viiper_connected);
                    if has_any_steam_viiper && !guard.binding_enforcer.is_active() {
                        guard.binding_enforcer.activate();
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

                let Some(device) = guard.devices.iter().find(|d| d.id == device_id) else {
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
