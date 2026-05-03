use std::mem::discriminant;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sdl3::event::Event;
use sdl3_sys::events::SDL_Event;
use sdl3_sys::joystick::SDL_JoystickID;

use crate::app::input::device::SDLDevice;
use crate::app::input::event::handler_events::InputHandlerEvent;
use crate::app::input::sdl_loop::{self, Subsystems};
use crate::app::input::sdl_utils::{get_gamepad_steam_handle, try_get_real_vid_pid_from_gamepad};
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::input::{
    context::Context,
    device::Device,
    event::router::{EventHandler, ListenEvent, RoutedEvent},
};
use crate::app::window;
use crate::config::get_config;

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

        if get_config()
            .controller_emulation
            .gyro_passthrough
            .unwrap_or(true)
            && let Some(gp) = &gamepad
        {
            unsafe {
                if gp.has_sensor(sdl3::sensor::SensorType::Gyroscope) {
                    if let Ok(()) = gp.sensor_set_enabled(sdl3::sensor::SensorType::Gyroscope, true)
                    {
                        tracing::debug!("Enabled gyroscope sensor on gamepad for SDL id {}", which);
                    } else {
                        tracing::error!(
                            "Failed to enable gyroscope sensor on gamepad for SDL id {}",
                            which
                        );
                    };
                }

                if gp.has_sensor(sdl3::sensor::SensorType::Accelerometer) {
                    if let Ok(()) =
                        gp.sensor_set_enabled(sdl3::sensor::SensorType::Accelerometer, true)
                    {
                        tracing::debug!(
                            "Enabled accelerometer sensor on gamepad for SDL id {}",
                            which
                        );
                    } else {
                        tracing::error!(
                            "Failed to enable accelerometer sensor on gamepad for SDL id {}",
                            which
                        );
                    };
                }
            }
        }

        let steam_handle = if let Some(gamepad) = &gamepad {
            get_gamepad_steam_handle(gamepad)
        } else {
            0
        };
        if get_config().steam.no_steam.unwrap_or(false) && steam_handle != 0 {
            tracing::debug!(
                "Ignoring steam handle {:016x} for SDL id {} due to no_steam config",
                steam_handle,
                which
            );
                    window::event::request_redraw();

            return;
        }

        let require_controllers_connected_before_launch = get_config()
            .controller_emulation
            .require_controllers_connected_before_launch
            .unwrap_or(true);

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        if require_controllers_connected_before_launch {
            let Ok(mut first_controller_detected_at) = ctx.first_controller_detected_at.lock()
            else {
                tracing::error!("Failed to lock first_controller_detected_at mutex");
                return;
            };
            if first_controller_detected_at.is_none() {
                *first_controller_detected_at = Some(std::time::Instant::now());
            } else {
                // fuck clippy
                if first_controller_detected_at.unwrap().elapsed().as_secs() >= 1 {
                    tracing::info!(
                        "Ignoring controller connection for SDL id {} due to require_controllers_connected_before_launch and time elapsed...",
                        which
                    );
                            window::event::request_redraw();

                    return;
                }
            }
            drop(first_controller_detected_at);
        }

        let delay_create = if require_controllers_connected_before_launch {
            let Ok(first_time) = ctx.first_controller_detected_at.lock() else {
                tracing::error!("Failed to lock first_controller_detected_at mutex");
                return;
            };
            first_time.map(|instant| Duration::from_secs(1).saturating_sub(instant.elapsed()))
        } else {
            None
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
                        .push_custom_event(InputHandlerEvent::IgnoreDevice { device_id: d.id })
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
                        window::event::request_redraw();

                return;
            }
        }

        let Some(device_mtx) = ctx.device_for_sdl_id(which) else {
            let device_id = ctx
                .next_device_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let device_type = get_config()
                .controller_emulation
                .default_controller_type
                .unwrap_or_default()
                .as_str()
                .to_string();
            let device = Arc::new(Mutex::new(Device {
                id: device_id,
                sdl_devices: vec![SDLDevice::new(which, joystick, gamepad)],
                steam_handle,
                viiper_type: Some(device_type.clone()),

                viiper_device: None,
                corresponding_device_id: None,
            }));
            if steam_handle != 0 {
                let Ok(mut device) = device.lock() else {
                    tracing::error!("Failed to lock new Device mutex");
                    return;
                };
                let corresponding_device_id = device.non_steam_sdl_id(&ctx);
                if let Some(corresponding_device_id) = corresponding_device_id {
                    tracing::info!(
                        "Device id {} (probably) corresponds to non-steam device id {}",
                        device_id,
                        corresponding_device_id
                    );
                    device.corresponding_device_id = Some(corresponding_device_id);
                    let non_steam_device = ctx.device_for_id(corresponding_device_id);
                    if let Some(non_steam_device) = non_steam_device {
                        let Ok(mut non_steam_device) = non_steam_device.lock() else {
                            tracing::error!(
                                "Failed to lock non-steam device id {}",
                                corresponding_device_id
                            );
                            return;
                        };
                        non_steam_device.corresponding_device_id = Some(device_id);
                    }
                }
            }
            ctx.devices.insert(device_id, device.clone());
            tracing::info!("Added new device {:?}", device.clone().lock().ok());

            if steam_handle != 0 || get_config().steam.no_steam.unwrap_or(false) {
                let viiper_bridge = self.viiper_bridge.clone();

                std::thread::spawn(move || {
                    if let Some(remaining) = delay_create
                        && remaining > Duration::ZERO
                    {
                        std::thread::sleep(remaining);
                    }

                    let Ok(viiper) = viiper_bridge.lock() else {
                        tracing::error!("Failed to lock ViiperBridge mutex");
                        return;
                    };
                    viiper.create_device(device_id, device_type.as_str());
                });
            }
            window::event::request_redraw();

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
        } else {
            let corresponding_device_id = device.non_steam_sdl_id(&ctx);
            if let Some(corresponding_device_id) = corresponding_device_id {
                tracing::info!(
                    "Device id {} (probably) corresponds to non-steam device id {}",
                    device.id,
                    corresponding_device_id
                );
                device.corresponding_device_id = Some(corresponding_device_id);
                let non_steam_device = ctx.device_for_id(corresponding_device_id);
                if let Some(non_steam_device) = non_steam_device {
                    let Ok(mut non_steam_device) = non_steam_device.lock() else {
                        tracing::error!(
                            "Failed to lock non-steam device id {}",
                            corresponding_device_id
                        );
                        return;
                    };
                    non_steam_device.corresponding_device_id = Some(device.id);
                }
            }
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

        if (device.steam_handle != 0 || !get_config().steam.no_steam.unwrap_or(false))
            && device.viiper_device.is_none()
        {
            let default_device_type = get_config()
                .controller_emulation
                .default_controller_type
                .unwrap_or_default()
                .as_str()
                .to_string();

            let viiper_bridge = self.viiper_bridge.clone();
            let device_id = device.id;
            let viiper_type = device.viiper_type.clone().unwrap_or(default_device_type);

            std::thread::spawn(move || {
                if let Some(remaining) = delay_create
                    && remaining > Duration::ZERO
                {
                    std::thread::sleep(remaining);
                }
                let Ok(viiper) = viiper_bridge.lock() else {
                    tracing::error!("Failed to lock ViiperBridge mutex");
                    return;
                };
                viiper.create_device(device_id, viiper_type.as_str());
            });
        }
        window::event::request_redraw();
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
