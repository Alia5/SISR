use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    event::WindowRunnerEvent, handler::router::EventHandler, runner::WindowRunner,
};

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        runner: &mut WindowRunner,
        _event_loop: &ActiveEventLoop,
        event: &WindowRunnerEvent,
    ) {
        tracing::trace!("RedrawHandler received event: {:?}", event);
        if let Some(window) = &runner.get_window() {
            window.request_redraw();
        }
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::Redraw())]
    }
}
