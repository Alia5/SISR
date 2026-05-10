use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    event::WindowRunnerEvent, handler::router::EventHandler, runner::WindowRunner,
};

#[derive(Default)]
pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        runner: &mut WindowRunner,
        _event_loop: &ActiveEventLoop,
        event: &WindowRunnerEvent,
    ) {
        tracing::trace!("SetFullscreenHandler received event: {:?}", event);

        let fullscreen = match event {
            WindowRunnerEvent::SetFullscreen(f) => *f,
            _ => return,
        };
        runner.set_fullscreen(fullscreen);
        crate::config::update_config(|c| {
            c.window.fullscreen = Some(fullscreen);
            c.window.create = Some(fullscreen);
        });
        runner.set_continuous_draw(fullscreen);
        runner.recalculate_passthrough();
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::SetFullscreen(true))]
    }
}
