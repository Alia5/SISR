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
        tracing::trace!("InvalidateSvelteStateHandler received event: {:?}", event);
        let Some(webview) = runner.get_webview_mut() else {
            tracing::warn!("No webview to reload.");
            return;
        };
        webview.invalidate_svelte_state();
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::InvalidateSvelteState())]
    }
}
