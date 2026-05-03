use sdl3_sys::events::SDL_Event;
use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use crate::app::input::context::Context;
use crate::app::input::device::Device;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::config::get_config;
use sdl3::event::Event;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
    _gyro_passthrough: bool,
    no_steam: bool,
}

impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>, viiper_bridge: Arc<Mutex<ViiperBridge>>) -> Self {
        Self {
            ctx,
            viiper_bridge,
            _gyro_passthrough: get_config()
                .controller_emulation
                .gyro_passthrough
                .unwrap_or(true),
            no_steam: get_config().steam.no_steam.unwrap_or(false),
        }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        // let event_type = SDL_EventType(unsafe { sdl_event.r#type });
        // tracing::trace!(event = ?event_type); // TODO: log only if enabled via flag
        let (which, sensor, data) = match event {
            Some(RoutedEvent::SdlEvent(event)) => match event {
                Event::ControllerSensorUpdated {
                    timestamp: _,
                    which,
                    sensor,
                    data,
                } => (*which, sensor, data),
                _ => {
                    tracing::warn!("Received non-sensor-updated event ");
                    return;
                }
            },
            _ => {
                tracing::warn!("Received non-SDL event ");
                return;
            }
        };

        // TODO: for steamdeck (check steam controller and others as well)
        // a hid report has to be sent to enable getting IMU data... -.-
        // TODO: probably switch to a custom SDL3 build... -.-
        // ouh boy...

        // TODO:
        // for switch controllers this works, though
        // worry about other crap (SteamHW, Don't have playstation controllers) later

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        let Some(device) = ctx.device_for_sdl_id(which) else {
            return;
        };
        let Ok(device) = device.lock() else {
            tracing::error!("Failed to lock Device mutex");
            return;
        };

        let mut maybe_sd_arc: Option<Arc<Mutex<Device>>> = None;
        let mut device = if device.steam_handle == 0 {
            let Some(id) = device.corresponding_device_id else {
                // tracing::warn!(
                //     "No corresponding device id found for SDL id {} in device id {}",
                //     which,
                //     device.id
                // );
                return;
            };
            drop(device);
            maybe_sd_arc = ctx.device_for_id(id);
            let Some(steam_dev) = &maybe_sd_arc else {
                // tracing::error!("Failed to find steam Device for device id {}", device.id);
                return;
            };
            let Ok(steam_dev) = steam_dev.lock() else {
                // tracing::error!("Failed to lock steam Device mutex");
                return;
            };
            steam_dev
        } else {
            device
        };
        drop(ctx);

        let device_id = device.id;
        let Some(viiper_device) = device.viiper_device.as_mut() else {
            // tracing::trace!(
            //     "No Viiper device found for SDL id {} in device id {}",
            //     which,
            //     device.id
            // );
            return;
        };

        if viiper_device.state.viiper_type() != Some(viiper_device.device.r#type.as_str()) {
            tracing::warn!(
                "Viiper device state type mismatch for device id {}. Reinitializing state.",
                device_id
            );
            viiper_device.init_state();
        }
        if viiper_device.device.r#type == "xbox360" {
            return;
        }

        match sensor {
            sdl3::sensor::SensorType::Gyroscope => {
                viiper_device.state.update_sensor(*sensor, data);
            }
            sdl3::sensor::SensorType::Accelerometer => {
                viiper_device.state.update_sensor(*sensor, data);
            }
            _ => {
                tracing::warn!(
                    "Unsupported sensor type {:?} for SDL id {} in device id {}",
                    sensor,
                    which,
                    device_id
                );
                return;
            }
        }

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };
        let Some(viiper_device_state_boxed) = viiper_device.state.boxed() else {
            tracing::error!("Failed to get boxed state for device id {}", device.id);
            return;
        };
        viiper.update_device_state_boxed(device_id, viiper_device_state_boxed);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![
            // ListenEvent::SdlEventType(SDL_EVENT_GAMEPAD_UPDATE_COMPLETE)
            ListenEvent::SdlEvent(discriminant(&Event::ControllerSensorUpdated {
                timestamp: 0,
                which: 0,
                sensor: sdl3::sensor::SensorType::Gyroscope,
                data: [0.0, 0.0, 0.0],
            })),
        ]
    }
}
