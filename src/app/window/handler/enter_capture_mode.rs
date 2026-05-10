use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    self, event::WindowRunnerEvent, handler::router::EventHandler, runner::WindowRunner
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
        tracing::trace!("EnterCaptureModeHandler received event: {:?}", event);

         let Some(wv) = runner.get_webview_mut() else {
            tracing::warn!("Webview not initialized");
            return;
        };
        wv.hide();
        runner.recalculate_passthrough();
        runner.update_cursor_visibility();
        window::event::request_redraw();
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::EnterCaptureMode())]
    }
}
