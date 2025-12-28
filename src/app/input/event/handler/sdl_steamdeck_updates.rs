use std::sync::{Arc, Mutex};

use sdl3_sys::events::{SDL_Event, SDL_EventType};

use crate::{
    app::input::{
        context::Context,
        device::DeviceState,
        device_update::steamdeck,
        event::router::{EventHandler, ListenEvent, RoutedEvent},
        sdl_loop::Subsystems,
        sdl_sisr::{SDL_EVENT_JOYSTICK_RAW_INPUT, SDL_JoyRawEvent},
        viiper_bridge::ViiperBridge,
    },
    config::get_config,
};

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
        let eraw = unsafe { &*(sdl_event as *const SDL_Event as *const SDL_JoyRawEvent) };
        let which = eraw.which;
        let raw_data = &eraw.data;

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        if Some(false) == get_config().steamdeck_gamepad_direct_forward {
            tracing::debug!(
                "Ignoring Steam Deck gamepad update for SDL id {} due to config",
                which
            );
            return;
        }

        let Some(device_mtx) = ctx.device_for_sdl_id(which) else {
            tracing::warn!("No device found for SDL id {}", which);
            return;
        };
        drop(ctx);
        let Ok(mut device) = device_mtx.lock() else {
            tracing::error!("Failed to lock Device mutex for SDL id {}", which);
            return;
        };
        let device = &mut *device;
        let Some(viiper_device) = device.viiper_device.as_mut() else {
            tracing::warn!(
                "No Viiper device found for SDL id {} in device id {}",
                which,
                device.id
            );
            return;
        };
        if device.viiper_type != Some("steamdeck".to_string()) {
            return;
        }

        if viiper_device.state.viiper_type() != Some(viiper_device.device.r#type.as_str()) {
            tracing::warn!(
                "Viiper device state type mismatch for device id {}. Reinitializing state.",
                device.id
            );
            viiper_device.init_state();
        }

        let DeviceState::SteamDeck {
            input_state: ref mut steamdeck_state,
            ..
        } = viiper_device.state
        else {
            tracing::error!("Device state is not SteamDeck type");
            return;
        };

        if !steamdeck::update_from_raw_event(steamdeck_state, raw_data) {
            return;
        }

        let Ok(viiper) = self.viiper_bridge.lock() else {
            tracing::error!("Failed to lock ViiperBridge mutex");
            return;
        };
        let Some(viiper_device_state_boxed) = viiper_device.state.boxed() else {
            tracing::error!("Failed to get boxed state for device id {}", device.id);
            return;
        };
        tracing::trace!("Updating VIIPER steamdeck state");
        viiper.update_device_state_boxed(device.id, viiper_device_state_boxed);
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEventType(SDL_EventType(
            SDL_EVENT_JOYSTICK_RAW_INPUT,
        ))]
    }
}
