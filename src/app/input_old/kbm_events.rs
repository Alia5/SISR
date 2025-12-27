#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KbmKeyEvent {
    pub scancode: u16,
    pub down: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KbmPointerEvent {
    pub dx: f32,
    pub dy: f32,

    pub wheel_y: f32,
    pub wheel_x: f32,

    pub button: u8,
    pub button_down: bool,
}

impl KbmPointerEvent {
    pub fn motion(dx: f32, dy: f32) -> Self {
        Self {
            dx,
            dy,
            wheel_x: 0.0,
            wheel_y: 0.0,
            button: 0,
            button_down: false,
        }
    }

    pub fn wheel(wheel_x: f32, wheel_y: f32) -> Self {
        Self {
            dx: 0.0,
            dy: 0.0,
            wheel_x,
            wheel_y,
            button: 0,
            button_down: false,
        }
    }

    pub fn button(button: u8, down: bool) -> Self {
        Self {
            dx: 0.0,
            dy: 0.0,
            wheel_x: 0.0,
            wheel_y: 0.0,
            button,
            button_down: down,
        }
    }
}
