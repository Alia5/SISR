use std::mem::{Discriminant, discriminant};

use winit::event_loop::ActiveEventLoop;

use crate::app::window::{
    event::{WindowRunnerEvent, request_redraw},
    handler::router::EventHandler,
    runner::WindowRunner,
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
        tracing::trace!("HideWindowHandler received event: {:?}", event);

        let overlay_open = match event {
            WindowRunnerEvent::OverlayStateChanged(open) => *open,
            _ => return,
        };

        runner.set_overlay_open(overlay_open);
        if overlay_open {
            runner.set_webview_visible(false);
            runner.set_continuous_draw(true);
            runner.set_passthrough(false);
        } else {
            runner.restore_webview_visibility();
            runner.restore_continuous_draw();
            runner.restore_passthrough();
        }

        request_redraw();
    }

    fn listen_events(&self) -> Vec<Discriminant<WindowRunnerEvent>> {
        vec![discriminant(&WindowRunnerEvent::OverlayStateChanged(true))]
    }
}
