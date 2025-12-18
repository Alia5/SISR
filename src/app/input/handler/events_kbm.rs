use tracing::{error, warn};

use sdl3::keyboard::Scancode;
use viiper_client::devices::keyboard::constants as kb_const;
use viiper_client::devices::{keyboard, mouse};

use crate::app::input::kbm_events::{KbmKeyEvent, KbmPointerEvent};

use super::EventHandler;
use super::viiper_bridge::StreamCommand;

impl EventHandler {
    pub fn on_kbm_release_all(&mut self) {
        let Ok(mut guard) = self.state.lock() else {
            error!("Failed to lock state for KBM release-all");
            return;
        };
        if !guard.kbm_emulation_enabled {
            return;
        }

        let kbd_id = guard
            .devices
            .values()
            .find(|d| d.viiper_type == "keyboard" && d.viiper_connected)
            .map(|d| d.id);
        let mouse_id = guard
            .devices
            .values()
            .find(|d| d.viiper_type == "mouse" && d.viiper_connected)
            .map(|d| d.id);

        guard.kbm_keyboard_modifiers = 0;
        guard.kbm_keyboard_keys.clear();
        guard.kbm_mouse_buttons = 0;
        drop(guard);

        if let Some(kbd_id) = kbd_id {
            self.viiper.update_device_state(
                kbd_id,
                StreamCommand::SendKeyboardInput(keyboard::KeyboardInput {
                    modifiers: 0,
                    count: 0,
                    keys: Vec::new(),
                }),
            );
        }

        if let Some(mouse_id) = mouse_id {
            self.viiper.update_device_state(
                mouse_id,
                StreamCommand::SendMouseInput(mouse::MouseInput {
                    dx: 0,
                    dy: 0,
                    buttons: 0,
                    wheel: 0,
                    pan: 0,
                }),
            );
        }
    }

    pub fn on_kbm_key_event(&mut self, ev: KbmKeyEvent) {
        let Ok(mut guard) = self.state.lock() else {
            error!("Failed to lock state for KBM key event");
            return;
        };
        if !guard.kbm_emulation_enabled {
            return;
        }

        let kbd_id = guard
            .devices
            .values()
            .find(|d| d.viiper_type == "keyboard" && d.viiper_connected)
            .map(|d| d.id);

        let modifier_bit = match ev.scancode {
            x if x == Scancode::LCtrl as u16 => Some(kb_const::MOD_LEFT_CTRL),
            x if x == Scancode::LShift as u16 => Some(kb_const::MOD_LEFT_SHIFT),
            x if x == Scancode::LAlt as u16 => Some(kb_const::MOD_LEFT_ALT),
            x if x == Scancode::LGui as u16 => Some(kb_const::MOD_LEFT_GUI),
            x if x == Scancode::RCtrl as u16 => Some(kb_const::MOD_RIGHT_CTRL),
            x if x == Scancode::RShift as u16 => Some(kb_const::MOD_RIGHT_SHIFT),
            x if x == Scancode::RAlt as u16 => Some(kb_const::MOD_RIGHT_ALT),
            x if x == Scancode::RGui as u16 => Some(kb_const::MOD_RIGHT_GUI),
            _ => None,
        };

        if let Some(bit) = modifier_bit {
            if ev.down {
                guard.kbm_keyboard_modifiers |= bit;
            } else {
                guard.kbm_keyboard_modifiers &= !bit;
            }
        } else {
            let Ok(key) = u8::try_from(ev.scancode) else {
                warn!("KBM scancode out of range ({}); dropping", ev.scancode);
                return;
            };
            if ev.down {
                _ = guard.kbm_keyboard_keys.insert(key);
            } else {
                _ = guard.kbm_keyboard_keys.remove(&key);
            }
        }

        let modifiers = guard.kbm_keyboard_modifiers;
        let keys: Vec<u8> = guard.kbm_keyboard_keys.iter().copied().collect();
        let count = u8::try_from(keys.len()).unwrap_or(u8::MAX);
        drop(guard);

        let Some(kbd_id) = kbd_id else {
            warn!(
                "KB/M emulation enabled but no connected virtual keyboard device found; dropping input"
            );
            return;
        };

        self.viiper.update_device_state(
            kbd_id,
            StreamCommand::SendKeyboardInput(keyboard::KeyboardInput {
                modifiers,
                count,
                keys,
            }),
        );
    }

    pub fn on_kbm_pointer_event(&mut self, ev: KbmPointerEvent) {
        let Ok(mut guard) = self.state.lock() else {
            error!("Failed to lock state for KBM pointer event");
            return;
        };
        if !guard.kbm_emulation_enabled {
            return;
        }

        let mouse_id = guard
            .devices
            .values()
            .find(|d| d.viiper_type == "mouse" && d.viiper_connected)
            .map(|d| d.id);

        let dx = ev.dx.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        let dy = ev.dy.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        let wheel = ev.wheel_y.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        let pan = ev.wheel_x.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

        if ev.button != 0 {
            let max_button = u8::BITS as u8;
            if ev.button <= max_button {
                let shift = ev.button - 1;
                if let Some(mask) = 1u8.checked_shl(shift as u32) {
                    if ev.button_down {
                        guard.kbm_mouse_buttons |= mask;
                    } else {
                        guard.kbm_mouse_buttons &= !mask;
                    }
                }
            }
        }
        let buttons = guard.kbm_mouse_buttons;
        drop(guard);

        let Some(mouse_id) = mouse_id else {
            warn!(
                "KB/M emulation enabled but no connected virtual mouse device found; dropping input"
            );
            return;
        };

        self.viiper.update_device_state(
            mouse_id,
            StreamCommand::SendMouseInput(mouse::MouseInput {
                dx,
                dy,
                buttons,
                wheel,
                pan,
            }),
        );
    }
}
