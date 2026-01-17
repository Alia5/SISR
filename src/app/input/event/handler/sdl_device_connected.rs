use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3::event::Event;
use sdl3::joystick::JoystickId;
use sdl3_sys::events::SDL_Event;
use sdl3_sys::joystick::SDL_JoystickID;

use crate::app::input::device::SDLDevice;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::sdl_loop::{self, Subsystems};
use crate::app::input::sdl_utils::{get_gamepad_steam_handle, try_get_real_vid_pid_from_gamepad};
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::input::{
    context::Context,
    device::Device,
    event::router::{EventHandler, ListenEvent, RoutedEvent},
};

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
}

impl Handler {
    pub fn new(context: Arc<Mutex<Context>>, viiper_bridge: Arc<Mutex<ViiperBridge>>) -> Self {
        Self {
            ctx: context,
            viiper_bridge,
        }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        tracing::debug!(event = ?event);
        let (which, joystick, gamepad) = match event {
            Some(RoutedEvent::SdlEvent(event)) => match event {
                Event::ControllerDeviceAdded { which, .. } => (
                    *which,
                    None,
                    subsystems.gamepad.open(SDL_JoystickID(*which)).ok(),
                ),
                Event::JoyDeviceAdded { which, .. } => (
                    *which,
                    subsystems.joystick.open(SDL_JoystickID(*which)).ok(),
                    None,
                ),
                _ => {
                    tracing::warn!("Received non-device-added event ");
                    return;
                }
            },
            _ => {
                tracing::warn!("Received non-SDL event ");
                return;
            }
        };

        let steam_handle = if let Some(gamepad) = &gamepad {
            get_gamepad_steam_handle(gamepad)
        } else {
            0
        };

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        if let Some(gp) = &gamepad {
            let (real_vid, real_pid) = match try_get_real_vid_pid_from_gamepad(gp) {
                Some((vid, pid)) => (vid, pid),
                None => {
                    tracing::warn!(
                        "Failed to determine real VID/PID for SDL Gamepad ID {}",
                        which
                    );
                    ("unknown".to_string(), "unknown".to_string())
                }
            };
            tracing::debug!(
                "SDL Gamepad ID {} has real VID/PID {}/{}",
                which,
                real_vid,
                real_pid
            );
            let exisisting_with_vid_pid = ctx.devices.iter().find(|r| {
                let Ok(d) = r.value().lock() else {
                    tracing::error!("Failed to lock device mutex");
                    return false;
                };
                let Some(vd) = d.viiper_device.as_ref() else {
                    return false;
                };
                vd.device.vid.to_lowercase() == real_vid && vd.device.pid.to_lowercase() == real_pid
            });
            if let Some(exisisting_with_vid_pid) = exisisting_with_vid_pid {
                let Ok(d) = exisisting_with_vid_pid.value().lock() else {
                    tracing::error!("Failed to lock device mutex");
                    return;
                };
                if d.sdl_devices.is_empty() {
                    _ = sdl_loop::get_event_sender()
                        .push_custom_event(HandlerEvent::IgnoreDevice { device_id: d.id })
                        .inspect_err(|e| {
                            tracing::error!(
                                "Failed to send ignore device event for ignored gamepad {}; {}",
                                which,
                                e
                            );
                        });
                }
                tracing::info!(
                    "Ignoring SDL device connection for SDL id {} due to existing VIIPER device",
                    which
                );
                return;
            }
        }

        let Some(device_mtx) = ctx.device_for_sdl_id(which) else {
            let device_id = ctx
                .next_device_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let device_type = "xbox360".to_string(); // TODO: determine viiper type by config (not implemented)
            let device = Arc::new(Mutex::new(Device {
                id: device_id,
                sdl_devices: vec![SDLDevice::new(which, joystick, gamepad)],
                steam_handle,
                viiper_type: Some(device_type.clone()), // TODO: determine viiper type by config (not implemented)

                viiper_device: None,
            }));
            ctx.devices.insert(device_id, device.clone());
            tracing::info!("Added new device {:?}", device.clone().lock().ok());

            if steam_handle != 0 {
                let Ok(viiper) = self.viiper_bridge.lock() else {
                    tracing::error!("Failed to lock ViiperBridge mutex");
                    return;
                };
                viiper.create_device(device_id, device_type.as_str());
            }

            return;
        };

        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock {:?}", device_mtx);
            return;
        };

        if device.steam_handle == 0 {
            tracing::info!(
                "Updating device {:?} with steam handle {:016x}",
                device,
                steam_handle
            );
            device.steam_handle = steam_handle;
        }
        let Some(sdl_device) = device.sdl_devices.iter_mut().find(|d| d.id == which) else {
            tracing::warn!(
                "WTF: device_for_sdl_id returned device without SDL id {}",
                which
            );
            return;
        };
        if joystick.is_some() {
            sdl_device.joystick = joystick;
        }
        if gamepad.is_some() {
            sdl_device.gamepad = gamepad;
        }
        sdl_device.update_info();
        tracing::info!("Added SDL id {} to existing device {:?}", which, device);

        if device.steam_handle != 0 && device.viiper_device.is_none() {
            let Ok(viiper) = self.viiper_bridge.lock() else {
                tracing::error!("Failed to lock ViiperBridge mutex");
                return;
            };
            viiper.create_device(
                device.id,
                device
                    .viiper_type
                    .clone()
                    .unwrap_or("xbox360".to_string())
                    .as_str(),
            );
        }
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![
            ListenEvent::SdlEvent(discriminant(&Event::ControllerDeviceAdded {
                timestamp: 0,
                which: 0,
            })),
            // ignore for now...
            // ListenEvent::SdlEvent(discriminant(&Event::JoyDeviceAdded {
            //     timestamp: 0,
            //     which: 0,
            // })),
        ]
    }
}
