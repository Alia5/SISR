use std::mem::discriminant;
use std::sync::{Arc, Mutex};
use std::thread;

use sdl3::event::Event;
use sdl3_sys::events::SDL_Event;

use crate::app::input::device::SDLDevice;
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::{
    context::Context,
    device::Device,
    event::router::{EventHandler, ListenEvent, RoutedEvent},
};
use crate::app::window;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
}
impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>) -> Self {
        Self { ctx }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        let event = match event {
            Some(RoutedEvent::SdlEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-SDL event ");
                return;
            }
        };
        let (which, button) = match event {
            Event::ControllerButtonDown { which, button, .. } => (*which, button),
            _ => {
                tracing::warn!("Received non-ControllerButtonDown event ");
                return;
            }
        };

        if *button == sdl3::gamepad::Button::Guide {
            // draw frames for a a second for overlay-spawn...
            tracing::debug!("HACK: Rending for a second to allow Steam overlay to spawn...");
            thread::spawn(|| {
                for _ in 0..60 {
                    thread::sleep(std::time::Duration::from_millis(16));
                    window::request_redraw();
                }
            });
        }

        // trigger only on A-down, while LB+RB+Back are held.
        if *button != sdl3::gamepad::Button::South {
            return;
        }

        let Ok(ctx) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };

        let Some(device) = ctx.device_for_sdl_id(which) else {
            return;
        };
        drop(ctx);

        let Ok(device) = device.lock() else {
            tracing::error!("Failed to lock Device mutex for SDL id {}", which);
            return;
        };

        let Some(gp) = device.sdl_devices.iter().find_map(|d| {
            if d.id == which && d.gamepad.is_some() {
                d.gamepad.as_ref()
            } else {
                None
            }
        }) else {
            return;
        };

        if gp.button(sdl3::gamepad::Button::LeftShoulder)
            && gp.button(sdl3::gamepad::Button::RightShoulder)
            && gp.button(sdl3::gamepad::Button::Back)
        {
            tracing::trace!("UI toggle controller chord detected on SDL ID {}", which);
            match window::get_event_sender().send_event(crate::app::window::RunnerEvent::ToggleUi())
            {
                Ok(_) => tracing::debug!("Successfully sent ToggleUi event to window"),
                Err(e) => tracing::error!("Failed to send ToggleUi to window: {e}"),
            }
        }
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::SdlEvent(discriminant(
            &Event::ControllerButtonDown {
                timestamp: 0,
                which: 0,
                button: sdl3::gamepad::Button::South,
            },
        ))]
    }
}
