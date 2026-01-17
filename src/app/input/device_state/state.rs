use std::any::Any;

use viiper_client::devices::{
    dualshock4::{Dualshock4Input, Dualshock4Output},
    keyboard::{KeyboardInput, KeyboardOutput},
    mouse::MouseInput,
    xbox360::{Xbox360Input, Xbox360Output},
};

use crate::app::input::device_state::{dualshock4 as dualshock4_state, xbox360 as xbox360_state};

#[derive(Clone, Default, Debug)]
pub enum DeviceState {
    #[default]
    Empty,
    Xbox360 {
        input_state: Xbox360Input,
        output_state: Xbox360Output,
    },
    Dualshock4 {
        input_state: Dualshock4Input,
        output_state: Dualshock4Output,
    },
    Keyboard {
        input_state: KeyboardInput,
        output_state: KeyboardOutput,
    },
    Mouse {
        input_state: MouseInput,
    },
}

impl DeviceState {
    pub fn is_empty(&self) -> bool {
        matches!(self, DeviceState::Empty)
    }

    pub fn viiper_type(&self) -> Option<&'static str> {
        match self {
            DeviceState::Xbox360 { .. } => Some("xbox360"),
            &DeviceState::Dualshock4 { .. } => Some("dualshock4"),
            DeviceState::Keyboard { .. } => Some("keyboard"),
            DeviceState::Mouse { .. } => Some("mouse"),
            DeviceState::Empty => None,
        }
    }

    pub fn boxed(&self) -> Option<Box<dyn Any + Send>> {
        match self {
            DeviceState::Xbox360 { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Dualshock4 { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Keyboard { input_state, .. } => Some(Box::new(input_state.clone())),
            DeviceState::Mouse { input_state } => Some(Box::new(input_state.clone())),
            DeviceState::Empty => None,
        }
    }

    pub fn update_from_sdl_gamepad(&mut self, gp: &sdl3::gamepad::Gamepad) {
        match self {
            DeviceState::Empty => {
                tracing::error!(
                    "Attempted to update Empty controller state from SDL gamepad: {:?}",
                    self
                );
            }
            DeviceState::Xbox360 { input_state, .. } => {
                xbox360_state::update_from_sdl_gamepad(input_state, gp);
            }
            DeviceState::Dualshock4 { input_state, .. } => {
                dualshock4_state::update_from_sdl_gamepad(input_state, gp);
            }
            _ => {
                tracing::warn!(
                    "Attempted not yet implemented controller update: {:?}",
                    self
                );
            }
        }
    }
}
