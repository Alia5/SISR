use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use egui::{Id, Vec2};
use sdl3::event::EventSender;

use crate::app::input::handler::{HandlerEvent, State};

pub fn draw(
    state: &mut State,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
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
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                // Use the live connection flag — `viiper_device` may still be cached even if the
                // server disconnected (or we hit network issues).
                let connected = state.devices.values().any(|d| d.viiper_connected);
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
                    .values()
                    .filter_map(|d| d.viiper_device.as_ref().map(|v| v.bus_id))
                    .collect::<BTreeSet<u32>>()
                    .into_iter()
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

                let is_localhost = state
                    .viiper_address
                    .map(|addr| addr.ip().is_loopback())
                    .unwrap_or(false);

                ui.separator();
                ui.add_enabled_ui(!is_localhost, |ui| {
                    let mut enabled = state.kbm_emulation_enabled;
                    if ui
                        .checkbox(&mut enabled, "Keyboard/mouse emulation")
                        .changed()
                        && let Ok(guard) = sdl_waker.lock()
                        && let Some(sender) = guard.as_ref()
                    {
                        _ = sender
                            .push_custom_event(HandlerEvent::SetKbmEmulationEnabled { enabled });
                    }
                    if is_localhost {
                        ui.label(
                            egui::RichText::new(
                                "KB/M emulation is only required / possible in networked setups.",
                            )
                            .weak()
                            .small(),
                        );
                    }
                    ui.label(
                        egui::RichText::new(
                            "KB/M emulation required the SISR window to be in focus",
                        )
                        .weak()
                        .small(),
                    );

                    if state.kbm_emulation_enabled {
                        ui.separator();
                        ui.label(egui::RichText::new("KB/M VIIPER devices").strong());

                        for (label, wanted_type) in
                            [("⌨ Keyboard", "keyboard"), ("🖱 Mouse", "mouse")]
                        {
                            let dev = state
                                .devices
                                .values()
                                .find(|d| d.viiper_type == wanted_type);
                            egui::CollapsingHeader::new(label)
                                .default_open(true)
                                .id_salt(format!("kbm_viiper_{wanted_type}"))
                                .show(ui, |ui| {
                                    let Some(dev) = dev else {
                                        ui.label(egui::RichText::new("Not present").weak());
                                        return;
                                    };

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(egui::RichText::new("Connected:").strong());
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}",
                                                dev.viiper_connected
                                            ))
                                            .weak(),
                                        );
                                    });

                                    match &dev.viiper_device {
                                        Some(viiper_dev) => {
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(egui::RichText::new("Bus ID:").strong());
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{}",
                                                        viiper_dev.bus_id
                                                    ))
                                                    .weak(),
                                                );
                                            });
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(
                                                    egui::RichText::new("Device ID:").strong(),
                                                );
                                                ui.label(
                                                    egui::RichText::new(
                                                        viiper_dev.dev_id.to_string(),
                                                    )
                                                    .weak(),
                                                );
                                            });
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(egui::RichText::new("Type:").strong());
                                                ui.label(
                                                    egui::RichText::new(
                                                        viiper_dev.r#type.to_string(),
                                                    )
                                                    .weak(),
                                                );
                                            });
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(
                                                    egui::RichText::new("Vendor ID:").strong(),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{:?}",
                                                        viiper_dev.vid
                                                    ))
                                                    .weak(),
                                                );
                                            });
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(
                                                    egui::RichText::new("Product ID:").strong(),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{:?}",
                                                        viiper_dev.pid
                                                    ))
                                                    .weak(),
                                                );
                                            });
                                        }
                                        None => {
                                            ui.label(egui::RichText::new("Not connected").weak());
                                        }
                                    }
                                });
                        }
                    }
                });
            })
        });
}
