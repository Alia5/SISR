use std::sync::{Arc, Mutex};

use egui::{Id, Vec2};
use sdl3::event::EventSender;

use crate::app::input::handler::State;

pub fn draw(
    state: &mut State,
    _: Arc<Mutex<Option<EventSender>>>,
    ctx: &egui::Context,
    open: &mut bool,
) {
    egui::Window::new("🐍 VIIPER")
        .id(Id::new("viiper_info"))
        .default_pos(ctx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_size(Vec2::new(360.0, 240.0))
        .collapsible(false)
        .resizable(true)
        .open(open)
        .show(ctx, |ui| {
            let connected = state.devices.iter().any(|(_, d)| d.viiper_device.is_some());
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Connected:").strong());
                ui.label(egui::RichText::new(if connected { "Yes" } else { "No" }).weak());
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("VIIPER Address:").strong());
                ui.label(
                    egui::RichText::new(
                        state
                            .viiper_address
                            .map(|addr| addr.to_string())
                            .unwrap_or("None".to_string()),
                    )
                    .weak(),
                );
            });

            let busses = state
                .devices
                .iter()
                .filter_map(|(_, d)| d.viiper_device.as_ref().map(|v| v.bus_id))
                .collect::<Vec<u32>>()
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>();

            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Bus IDs:").strong());
                ui.label(
                    egui::RichText::new(if busses.is_empty() {
                        "None".to_string()
                    } else {
                        busses.join(", ")
                    })
                    .weak(),
                );
            });
        });
}
