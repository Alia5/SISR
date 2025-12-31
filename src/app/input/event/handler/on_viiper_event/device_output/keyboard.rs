use std::sync::{Arc, Mutex};

use crate::app::input::context::Context;

pub fn handle_output(ctx: Arc<Mutex<Context>>, device_id: &u64, leds: &u8) {
    // TODO: set leds on host
    tracing::warn!("Ignoreing Keyboard output! Not implemented");
}
