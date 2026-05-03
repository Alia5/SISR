use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    event::WindowRunnerEvent, handler::router::EventHandler, runner::WindowRunner,
};

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _runner: &mut WindowRunner,
        event_loop: &ActiveEventLoop,
        event: &WindowRunnerEvent,
    ) {
        tracing::trace!("QuitHandler received event: {:?}", event);
        event_loop.exit();
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::Quit())]
    }
}
