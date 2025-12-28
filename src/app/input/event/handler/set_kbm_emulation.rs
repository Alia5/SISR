use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::gui::dialogs::{self, Dialog, push_dialog};
use crate::app::input::context::Context;
use crate::app::input::device::Device;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::kbm_context::KbmContext;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::ViiperBridge;
use crate::app::window::{self, RunnerEvent};

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
    kbm_ctx: Arc<Mutex<KbmContext>>,
}
impl Handler {
    pub fn new(
        ctx: Arc<Mutex<Context>>,
        viiper_bridge: Arc<Mutex<ViiperBridge>>,
        kbm_ctx: Arc<Mutex<KbmContext>>,
    ) -> Self {
        Self {
            ctx,
            viiper_bridge,
            kbm_ctx,
        }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        tracing::debug!(event = ?event);
        let event = match event {
            Some(RoutedEvent::UserEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-handler event ");
                return;
            }
        };
        let (enabled, initialize) = match event {
            HandlerEvent::SetKbmEmulation {
                enabled,
                initialize,
            } => (*enabled, *initialize),
            _ => {
                tracing::warn!("Received non-SetKbmEmulationEnabled event ");
                return;
            }
        };

        if !initialize && window::is_kbm_emulation_enabled() == enabled {
            tracing::info!(
                "KBM emulation already {}",
                if enabled { "enabled" } else { "disabled" }
            );
            return;
        }
        window::set_kbm_emulation_enabled(enabled);
        let Ok(mut context) = self.ctx.lock() else {
            tracing::error!("Failed to lock Context mutex");
            return;
        };
        context.keyboard_mouse_emulation = enabled;
        drop(context);
        tracing::info!("KBM emulation toggled: {}", enabled);

        if let Err(e) =
            window::get_event_sender().send_event(RunnerEvent::SetKbmCursorGrab(enabled))
        {
            tracing::warn!("Failed to notify window about KB/M cursor grab toggle: {e}");
        }

        let Ok(mut kbm_ctx) = self.kbm_ctx.lock() else {
            tracing::error!("Failed to lock KbmContext mutex");
            return;
        };

        kbm_ctx.keyboard_modifiers = 0;
        kbm_ctx.keyboard_keys.clear();
        kbm_ctx.mouse_buttons = 0;

        // When enabling, show a single OK dialog. On OK we enter capture mode.
        // except when ebaled via config/cli at startup (initialize=true)
        if enabled && !initialize {
            const TITLE: &str = "KB/M emulation";
            let already_open = dialogs::REGISTRY
                .get()
                .map(|r| r.snapshot_dialogs().iter().any(|d| d.title == TITLE))
                .unwrap_or(false);

            if !already_open {
                let msg = "KB/M emulation enabled.\n\n\
UI will be hidden and the cursor will be captured when you enter capture mode.\n\n\
Toggle UI/capture:\n\
  Keyboard: Ctrl+Shift+Alt+S\n\
  Gamepad:  LB+RB+Back+A";
                _ = push_dialog(Dialog::new_ok(TITLE, msg, move || {
                    if let Err(e) = window::get_event_sender()
                        .send_event(crate::app::window::RunnerEvent::EnterCaptureMode())
                    {
                        tracing::warn!("Failed to enter capture mode after KB/M OK: {e}");
                    }
                }))
                .inspect_err(|e| tracing::warn!("Failed to push KB/M emulation dialog: {e}"));
            }
        }

        if enabled {
            let Ok(ctx) = self.ctx.lock() else {
                tracing::error!("Failed to lock Context mutex");
                return;
            };
            let has_keyboard = ctx.devices.iter().any(|r| {
                let Ok(d) = r.value().lock() else {
                    return false;
                };
                d.viiper_type.as_deref() == Some("keyboard")
            });
            let has_mouse = ctx.devices.iter().any(|r| {
                let Ok(d) = r.value().lock() else {
                    return false;
                };
                d.viiper_type.as_deref() == Some("mouse")
            });

            let Ok(viiper) = self.viiper_bridge.lock() else {
                tracing::error!("Failed to lock ViiperBridge mutex");
                return;
            };

            if !has_keyboard {
                let keyboard_id = ctx
                    .next_device_id
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let keyboard_device = Device {
                    id: keyboard_id,
                    viiper_type: Some("keyboard".to_string()),
                    ..Default::default()
                };
                ctx.devices
                    .insert(keyboard_id, Arc::new(Mutex::new(keyboard_device)));
                viiper.create_device(keyboard_id, "keyboard");
                kbm_ctx.keyboard_id = Some(keyboard_id);
                tracing::debug!("Created virtual keyboard device with id {}", keyboard_id);
            }

            if !has_mouse {
                let mouse_id = ctx
                    .next_device_id
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mouse_device = Device {
                    id: mouse_id,
                    viiper_type: Some("mouse".to_string()),
                    ..Default::default()
                };
                ctx.devices
                    .insert(mouse_id, Arc::new(Mutex::new(mouse_device)));
                viiper.create_device(mouse_id, "mouse");
                kbm_ctx.mouse_id = Some(mouse_id);
                tracing::debug!("Created virtual mouse device with id {}", mouse_id);
            }
        } else {
            let Ok(guard) = self.ctx.lock() else {
                tracing::error!("Failed to lock Context mutex");
                return;
            };
            let kbm_ids: Vec<u64> = guard
                .devices
                .iter()
                .filter_map(|r| {
                    let Ok(d) = r.value().lock() else {
                        return None;
                    };
                    if d.viiper_type.as_deref() == Some("keyboard")
                        || d.viiper_type.as_deref() == Some("mouse")
                    {
                        Some(d.id)
                    } else {
                        None
                    }
                })
                .collect();
            let Ok(mut bridge) = self.viiper_bridge.lock() else {
                tracing::error!("Failed to lock ViiperBridge mutex");
                return;
            };
            for id in kbm_ids {
                bridge.remove_device(id);
                guard.devices.remove(&id);
            }
            kbm_ctx.mouse_id = None;
            kbm_ctx.keyboard_id = None;
        }

        window::request_redraw();
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![ListenEvent::HandlerEvent(discriminant(
            &HandlerEvent::SetKbmEmulation {
                enabled: false,
                initialize: false,
            },
        ))]
    }
}
