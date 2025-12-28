use std::mem::discriminant;

use sdl3_sys::events::SDL_Event;

use crate::app::core::get_tokio_handle;
use crate::app::gui::dialogs::{Dialog, push_dialog};
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::steam_utils::cef_debug;
use crate::app::steam_utils::util::launched_via_steam;
use crate::app::window;

pub struct Handler {}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        tracing::debug!(event = ?event);

        let cont_redraw = window::is_continuous_redraw();
        get_tokio_handle().spawn(async move {
            if !launched_via_steam() {
                tracing::debug!("NOT launched via Steam, delaying CEF overlay notifier injection");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                // TODO: FIXME!
            }
            match cef_debug::inject::inject(
                "Overlay",
                str::from_utf8(cef_debug::payloads::OVERLAY_STATE_NOTIFIER)
                    .expect("Failed to convert overlay notifier payload to string"),
            )
            .await
            {
                Ok(_) => tracing::info!("Successfully injected CEF overlay state notifier"),
                Err(e) => {
                    tracing::error!("Failed to inject CEF overlay state notifier: {}", e);
                    if cont_redraw {
                        _ = push_dialog(Dialog::new_ok(
                            "Failed to init Steam overlay notifier",
                            format!(
                                "SISR was not able to initialize the overlay notifier.
\nError: {}",
                                e
                            ),
                            move || {},
                        ));
                    } else {
                        _ = push_dialog(Dialog::new_yes_no(
                            "Failed to init Steam overlay notifier",
                            format!(
                                "SISR was not able to initialize the overlay notifier.
It is recommended you enable the \"Continuous Redraw\" option.
This can cause higher CPU/GPU usage.
Enable continuous redraw now?

\nError: {}",
                                e
                            ),
                            move || {
                                window::set_continuous_redraw(true);
                            },
                            || {},
                        ));
                    }
                }
            }
        });
        window::request_redraw();
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::CefDebugReady { port: 0 },
        ))]
    }
}
