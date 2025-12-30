use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3::event::Event;
use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::steamdeck_hid;
use crate::app::input::viiper_bridge::ViiperBridge;

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
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        tracing::debug!(event = ?event);
        let event = match event {
            Some(RoutedEvent::SdlEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-SDL event ");
                return;
            }
        };
        let which = match event {
            Event::ControllerDeviceRemoved { which, .. } => *which,
            Event::JoyDeviceRemoved { which, .. } => *which,
            _ => {
                tracing::warn!("Received non-device-removed event ");
                return;
            }
        };

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        let Some(device_mtx) = ctx.device_for_sdl_id(which) else {
            tracing::warn!("No device found for SDL id {}", which);
            return;
        };
        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock {:?}", device_mtx);
            return;
        };

        let is_steamdeck = device.viiper_type.as_deref() == Some("steamdeck");
        let Some(sdl_device) = device.sdl_device(which) else {
            tracing::warn!(
                "WTF No SDL device found for SDL id {} in device {:?}",
                which,
                device_mtx
            );
            ctx.devices.remove(&device.id);
            tracing::info!("Device {:?} removed", device_mtx);
            return;
        };

        match event {
            Event::ControllerDeviceRemoved { .. } => {
                if is_steamdeck
                    && let Some(gp) = sdl_device.gamepad.as_ref()
                    && let Some(path) = gp.path()
                {
                    steamdeck_hid::close_path(&path);
                }
                sdl_device.gamepad.take();
            }
            Event::JoyDeviceRemoved { .. } => {
                sdl_device.joystick.take();
            }
            _ => {
                // cannot happen
            }
        }

        if sdl_device.joystick.is_none() && sdl_device.gamepad.is_none() {
            device.sdl_devices.retain(|d| d.id != which);
        } else {
            sdl_device.update_info();
        }

        tracing::info!(
            "SDL device disconnected: SDL id {}, device {:?}",
            which,
            device_mtx
        );

        if device.sdl_devices.is_empty() {
            ctx.devices.remove(&device.id);
        }
        tracing::info!("Device {:?} removed", device_mtx);
        let Ok(mut bridge) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };
        bridge.remove_device(device.id);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![
            ListenEvent::SdlEvent(discriminant(&Event::ControllerDeviceRemoved {
                timestamp: 0,
                which: 0,
            })),
            ListenEvent::SdlEvent(discriminant(&Event::JoyDeviceRemoved {
                timestamp: 0,
                which: 0,
            })),
        ]
    }
}
