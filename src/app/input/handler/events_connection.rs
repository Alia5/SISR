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
    pub fn on_pad_added(&mut self, event: &Event) {
        let (which, is_joystick) = match event {
            Event::JoyDeviceAdded { which, .. } => (which, true),
            Event::ControllerDeviceAdded { which, .. } => (which, false),
            _ => {
                warn!("Unexpected event for pad addition: {:?}", event);
                return;
            }
        };

        trace!(
            "{} added with ID {}",
            if is_joystick { "Joystick" } else { "Gamepad" },
            which
        );

        let sdl_dev = if is_joystick {
            self.sdl_joystick.open(*which).ok().map(SDLDevice::Joystick)
        } else {
            self.sdl_gamepad.open(*which).ok().map(SDLDevice::Gamepad)
        };
        let Some(sdl_device) = sdl_dev else {
            warn!("Failed to open SDL device with ID {}", which);
            return;
        };

        let steam_handle = match &sdl_device {
            SDLDevice::Joystick(_) => 0,
            SDLDevice::Gamepad(p) => get_gamepad_steam_handle(p),
        };

        let sdl_device_info = SdlDeviceInfo::from(&sdl_device);
        self.sdl_devices.entry(*which).or_default().push(sdl_device);

        let Ok(mut guard) = self.state.lock() else {
            error!("Failed to lock state for adding device");
            return;
        };

        if let Some(&device_id) = self.sdl_id_to_device.get(which) {
            if let Some(existing_device) = guard.devices.iter_mut().find(|d| d.id == device_id) {
                existing_device.sdl_device_infos.push(sdl_device_info);
                debug!(
                    "Added extra SDL {} for device {}; SDL ID {}; Total SDL devices {}",
                    if is_joystick { "Joystick" } else { "Gamepad" },
                    device_id,
                    which,
                    existing_device.sdl_device_infos.len()
                );
                handle_existing_device_connect(
                    &mut self.viiper,
                    existing_device,
                    steam_handle,
                    *which,
                );
            }
        } else {
            let device_id = self.next_device_id;
            self.next_device_id += 1;
            self.sdl_id_to_device.insert(*which, device_id);

            handle_new_device(
                &mut self.viiper,
                &mut guard,
                device_id,
                *which,
                steam_handle,
                is_joystick,
                sdl_device_info,
            );
        }
    }
    pub fn on_pad_removed(&mut self, event: &Event) {
        match event {
            Event::JoyDeviceRemoved { which, .. }
            | Event::ControllerDeviceRemoved { which, .. } => {
                trace!(
                    "{} removed with ID {}",
                    if matches!(event, Event::JoyDeviceRemoved { .. }) {
                        "Joystick"
                    } else {
                        "Gamepad"
                    },
                    which
                );

                if let Some(devices) = self.sdl_devices.get_mut(which) {
                    _ = devices.pop();
                    if devices.is_empty() {
                        self.sdl_devices.remove(which);
                    }
                }

                if let Ok(mut guard) = self
                    .state
                    .lock()
                    .map_err(|e| error!("Failed to lock state for removing device: {}", e))
                {
                    let is_joystick = matches!(event, Event::JoyDeviceRemoved { .. });

                    let Some(&device_id) = self.sdl_id_to_device.get(which) else {
                        warn!("No device found for SDL ID {} in pad removal", which);
                        return;
                    };

                    let Some(device) = guard.devices.iter_mut().find(|d| d.id == device_id) else {
                        warn!(
                            "Device {} not found in state for SDL ID {}",
                            device_id, which
                        );
                        return;
                    };

                    let before_len = device.sdl_device_infos.len();

                    if let Some(idx) = device
                        .sdl_device_infos
                        .iter()
                        .position(|info| info.is_gamepad != is_joystick)
                    {
                        device.sdl_device_infos.remove(idx);
                        debug!(
                            "Removed SDL {} device info for device {}; SDL ID {}; Remaining SDL devices {}",
                            if is_joystick { "Joystick" } else { "Gamepad" },
                            device_id,
                            which,
                            device.sdl_device_infos.len()
                        );
                    } else if before_len > 0 {
                        warn!(
                            "Could not find matching SDL {} device info to remove for device {}",
                            if is_joystick { "Joystick" } else { "Gamepad" },
                            device_id
                        );
                    }

                    if device.sdl_device_infos.is_empty() {
                        device.sdl_ids.remove(which);

                        if device.sdl_ids.is_empty() {
                            let had_steam_viiper =
                                device.steam_handle != 0 && device.viiper_connected;

                            self.viiper.remove_device(device_id);
                            self.sdl_id_to_device.remove(which);
                            guard.devices.retain(|d| d.id != device_id);
                            info!(
                                "Removed device {} (last SDL {} with ID {})",
                                device_id,
                                if is_joystick { "Joystick" } else { "Gamepad" },
                                which
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
                }
            }
            _ => {
                warn!("Unexpected event for pad removal: {:?}", event);
            }
        }
    }

    // The high-level sdl3-rs `Event::Unknown` doesn't expose the `which` field from
    // `SDL_GamepadDeviceEvent`. We work around this by refreshing all tracked pads.
    //
    // See: https://github.com/libsdl-org/SDL/blob/main/include/SDL3/SDL_events.h#L672-L677
    pub fn on_steam_handle_updated(&mut self, _event: &Event) {
        let Ok(mut guard) = self.state.lock() else {
            warn!("Failed to lock state for steam handle update");
            return;
        };
        // should only be one bus for all devices, this is fine for now!
        self.sdl_devices
            .values()
            .flat_map(|d| {
                d.iter().filter_map(|dev| match dev {
                    SDLDevice::Gamepad(pad) => Some(pad),
                    _ => None,
                })
            })
            .for_each(|pad| {
                let Ok(instance_id) = pad.id() else {
                    return;
                };
                let steam_handle = get_gamepad_steam_handle(pad);

                // Look up our device ID for this SDL instance ID
                let Some(&device_id) = self.sdl_id_to_device.get(&instance_id) else {
                    return;
                };

                if let Some(device) = guard.devices.iter_mut().find(|d| d.id == device_id) {
                    if device.steam_handle != steam_handle {
                        device.steam_handle = steam_handle;
                        info!(
                            "Updated steam handle for device {} (SDL ID {}) to {}",
                            device_id, instance_id, steam_handle
                        );
                    }

                    if device.viiper_device.is_none() {
                        info!(
                            "Connecting device {} upon steam handle update with steam handle {}",
                            device_id, steam_handle
                        );
                        self.viiper.create_device(device);
                    }
                }
            });
        self.request_redraw();
    }
}

fn handle_existing_device_connect(
    viiper: &mut super::viiper_bridge::ViiperBridge,
    device: &mut Device,
    steam_handle: u64,
    sdl_id: u32,
) {
    if device.steam_handle == 0 && steam_handle != 0 {
        device.steam_handle = steam_handle;
        info!(
            "Updated steam handle for device {} (SDL ID {}) to {}",
            device.id, sdl_id, steam_handle
        );

        if device.viiper_device.is_some() {
            debug!(
                "Device {} already has a VIIPER device; skipping creation",
                device.id
            );
            return;
        }

        info!(
            "Connecting device {} upon connect with steam handle {}",
            device.id, steam_handle
        );
        viiper.create_device(device);
    }
}

fn handle_new_device(
    viiper: &mut super::viiper_bridge::ViiperBridge,
    guard: &mut std::sync::MutexGuard<'_, super::State>,
    device_id: u64,
    sdl_id: u32,
    steam_handle: u64,
    is_joystick: bool,
    sdl_device_info: SdlDeviceInfo,
) {
    let mut sdl_ids = std::collections::HashSet::new();
    sdl_ids.insert(sdl_id);

    let device = Device {
        id: device_id,
        sdl_ids,
        steam_handle,
        state: DeviceState::default(),
        sdl_device_infos: vec![sdl_device_info],
        ..Default::default()
    };
    if is_joystick {
        guard.devices.push(device);
        info!(
            "Added Joystick device {} (SDL ID {}); Steam Handle: {}",
            device_id, sdl_id, steam_handle
        );
        return;
    }

    if steam_handle != 0 {
        info!(
            "Connecting device {} (SDL ID {}) upon connect with steam handle {}",
            device_id, sdl_id, steam_handle
        );
        viiper.create_device(&device);
    }
    guard.devices.push(device);
}
