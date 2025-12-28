use std::sync::{Arc, Mutex};

use sdl3_sys::events::{SDL_EVENT_GAMEPAD_STEAM_HANDLE_UPDATED, SDL_Event, SDL_EventType};

use crate::app::input::context::Context;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::sdl_utils::get_gamepad_steam_handle;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::window;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
}
impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>, viiper_bridge: Arc<Mutex<ViiperBridge>>) -> Self {
        Self { ctx, viiper_bridge }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        _event: &Option<RoutedEvent>,
        sdl_event: &SDL_Event,
    ) {
        let event_type = SDL_EventType(unsafe { sdl_event.r#type });
        tracing::debug!(event = ?event_type);
        let which = unsafe { sdl_event.gdevice.which };

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        let Some(device_mtx) = ctx.device_for_sdl_id(which) else {
            tracing::warn!("No device found for SDL id {}", which);
            return;
        };
        drop(ctx);
        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock Device mutex for SDL id {}", which);
            return;
        };
        let Some(gp) = device.sdl_devices.iter_mut().find_map(|d| {
            if d.id == which && d.gamepad.is_some() {
                d.gamepad.as_mut()
            } else {
                None
            }
        }) else {
            tracing::warn!(
                "No gamepad found for SDL id {} in device id {}",
                which,
                device.id
            );
            return;
        };
        let steam_handle = get_gamepad_steam_handle(gp);
        device.steam_handle = steam_handle;
        tracing::info!(
            "Updated steam handle for device id {} (SDL id {}): {}",
            device.id,
            which,
            steam_handle
        );
        if device.viiper_device.is_none() && steam_handle != 0 {
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
        window::request_redraw();
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEventType(
            SDL_EVENT_GAMEPAD_STEAM_HANDLE_UPDATED,
        )]
    }
}
