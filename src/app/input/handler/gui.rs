use tracing::warn;

use crate::app::{input::sdl_device_info::SdlValue, window::RunnerEvent};

use super::{EventHandler, State};

impl EventHandler {
    pub(super) fn request_redraw(&self) {
        _ = self
            // the legend of zelda: the
            .winit_waker
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|p| p.send_event(RunnerEvent::Redraw())))
            .map(|r| r.inspect_err(|e| warn!("Failed to request GUI redraw: {}", e)));
    }

    pub(super) fn on_draw(state: &mut State, ctx: &egui::Context) {
        egui::Window::new("🎮 GamePads")
            .default_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Connected GamePads:");
                    for device in &state.devices {
                        ui.group(|ui| {
                            ui.label(format!("Device ID: {}", device.id));
                            ui.label(format!(
                                "SDL IDs: {}",
                                device
                                    .sdl_ids
                                    .iter()
                                    .map(|id| id.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                            ui.label(format!("Steam Handle: {}", device.steam_handle));
                            ui.label(format!(
                                "SDL Device Count: {}",
                                device.sdl_device_infos.len()
                            ));

                            for (idx, info) in device.sdl_device_infos.iter().enumerate() {
                                ui.collapsing(
                                    format!(
                                        "SDL {} #{}",
                                        if info.is_gamepad {
                                            "Gamepad"
                                        } else {
                                            "Joystick"
                                        },
                                        idx
                                    ),
                                    |ui| {
                                        Self::render_properties(ui, &info.properties);
                                    },
                                );
                            }

                            ui.collapsing("🐍 VIIPER Device", |ui| match &device.viiper_device {
                                Some(viiper_dev) => {
                                    ui.label(format!("Connected: {}", device.viiper_connected));
                                    ui.label(format!("Bus ID: {}", viiper_dev.bus_id));
                                    ui.label(format!("Device ID: {}", viiper_dev.dev_id));
                                    ui.label(format!("Type: {}", viiper_dev.r#type));
                                    ui.label(format!("Vendor ID: {:?}", viiper_dev.vid));
                                    ui.label(format!("Product ID: {:?}", viiper_dev.pid));
                                }
                                None => {
                                    ui.label("Not connected");
                                }
                            });
                        });
                    }
                });
            });
        egui::Window::new("🐍 VIIPER").show(ctx, |ui| {
            ui.label(format!(
                "VIIPER Address: {}",
                state
                    .viiper_address
                    .map(|addr| addr.to_string())
                    .unwrap_or("None".to_string())
            ));

            let busses = state
                .devices
                .iter()
                .filter_map(|d| d.viiper_device.as_ref().map(|v| v.bus_id))
                .collect::<Vec<u32>>()
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>();
            ui.label(format!(
                "Bus IDs: {}",
                if busses.is_empty() {
                    "None".to_string()
                } else {
                    busses.join(", ")
                }
            ));
        });

        egui::Window::new("🎮 Steam Bindings").show(ctx, |ui| {
            let enforcer = &mut state.binding_enforcer;

            ui.label(format!(
                "Game ID: {}",
                enforcer
                    .game_id()
                    .map(|id| id.to_string())
                    .unwrap_or("N/A".to_string())
            ));
            ui.label(format!(
                "App ID: {}",
                enforcer
                    .app_id()
                    .map(|id| id.to_string())
                    .unwrap_or("N/A".to_string())
            ));

            ui.separator();

            let has_app_id = enforcer.app_id().is_some();
            let mut active = enforcer.is_active();

            ui.add_enabled_ui(has_app_id, |ui| {
                if ui.checkbox(&mut active, "Enforce Bindings").changed() {
                    if active {
                        enforcer.activate();
                    } else {
                        enforcer.deactivate();
                    }
                }
            });
        });
    }

    fn render_properties(
        ui: &mut egui::Ui,
        properties: &std::collections::HashMap<String, SdlValue>,
    ) {
        // Sort keys for consistent display
        let mut keys: Vec<_> = properties.keys().collect();
        keys.sort();

        for key in keys {
            let value = &properties[key];
            match value {
                SdlValue::Nested(nested) => {
                    ui.collapsing(format!("📁 {}", key), |ui| {
                        Self::render_properties(ui, nested);
                    });
                }
                _ => {
                    ui.label(format!("{}: {}", key, value));
                }
            }
        }
    }
}
