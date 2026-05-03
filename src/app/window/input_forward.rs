use std::sync::Arc;

use winit::{
    event::WindowEvent, event_loop::ActiveEventLoop, keyboard::PhysicalKey, window::Window,
};

use crate::app::input::{event::handler_events::InputHandlerEvent, kbm_events, sdl_loop};

#[derive(Default)]
pub struct InputForward {
    modifiers: winit::keyboard::ModifiersState,
    last_cursor_pos: Option<(f64, f64)>,
}

impl InputForward {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_input(
        &mut self,
        window: &Option<Arc<Window>>,
        _event_loop: &ActiveEventLoop,
        event: &WindowEvent,
        capture_forward: bool,
    ) {
        match event {
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::ElementState;

                if matches!(event.state, ElementState::Pressed)
                    && let PhysicalKey::Code(code) = event.physical_key
                    && code == winit::keyboard::KeyCode::KeyS
                    && self.modifiers.control_key()
                    && self.modifiers.shift_key()
                    && self.modifiers.alt_key()
                {
                    tracing::trace!("Toggle UI keybinding pressed");
                    let Some(window) = window else {
                        return;
                    };
                    // if self.ui_visible {
                    //     self.ui_visible = false;
                    //     if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                    //         tracing::warn!("Failed to confine cursor to window: {e}");
                    //     }
                    // } else {
                    //     self.ui_visible = true;
                    //     _ = window.set_cursor_grab(CursorGrabMode::None);
                    //     self.try_push_kbm_event(HandlerEvent::KbmReleaseAll());
                    // }
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                self.last_cursor_pos = Some((x, y));
            }
            WindowEvent::CursorLeft { .. } => {
                self.last_cursor_pos = None;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::ElementState;

                if let Some(btn) = Self::map_mouse_button(button) {
                    let down = matches!(state, ElementState::Pressed);
                    if capture_forward {
                        self.try_push_kbm_event(InputHandlerEvent::KbmPointerEvent(
                            kbm_events::KbmPointerEvent::button(btn, down),
                        ));
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;

                let (wheel_x, wheel_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };

                if (wheel_x != 0.0 || wheel_y != 0.0) && capture_forward {
                    self.try_push_kbm_event(InputHandlerEvent::KbmPointerEvent(
                        kbm_events::KbmPointerEvent::wheel(wheel_x, wheel_y),
                    ));
                }
            }
            _ => {}
        }
    }

    fn map_mouse_button(button: &winit::event::MouseButton) -> Option<u8> {
        match button {
            winit::event::MouseButton::Left => Some(1),
            winit::event::MouseButton::Middle => Some(2),
            winit::event::MouseButton::Right => Some(3),
            winit::event::MouseButton::Back => Some(4),
            winit::event::MouseButton::Forward => Some(5),
            winit::event::MouseButton::Other(n) => u8::try_from(*n).ok(),
        }
    }

    fn try_push_kbm_event(&self, ev: InputHandlerEvent) {
        if let Err(e) = sdl_loop::get_event_sender().push_custom_event(ev) {
            tracing::trace!("Failed to push KBM custom event to SDL: {e}");
        }
    }
}
