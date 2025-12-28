use std::collections::BTreeSet;

use egui::{Id, Vec2};

use crate::app::input::{context::Context, event::handler_events::HandlerEvent, sdl_loop};

pub fn draw(ctx: &Context, ectx: &egui::Context, open: &mut bool) {
    egui::Window::new("🐍 VIIPER")
        .id(Id::new("viiper_info"))
        .default_pos(ectx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_size(Vec2::new(360.0, 240.0))
        .collapsible(false)
        .resizable(true)
        .open(open)
        .show(ectx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("VIIPER Address:").strong());
                    ui.label(
                        egui::RichText::new(
                            ctx.viiper_address
                                .map(|addr| addr.to_string())
                                .unwrap_or("None".to_string()),
                        )
                        .weak(),
                    );
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("VIIPER Available:").strong());
                    ui.label(
                        egui::RichText::new(if ctx.viiper_available {
                            "true"
                        } else {
                            "false"
                        })
                        .weak(),
                    );
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("VIIPER Version:").strong());
                    ui.label(
                        egui::RichText::new(ctx.viiper_version.as_deref().unwrap_or("")).weak(),
                    );
                });

                let connected = ctx.devices.iter().any(|r| {
                    let Ok(dev) = r.value().lock() else {
                        return false;
                    };
                    dev.viiper_device.is_some()
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Any device(s) connected:").strong());
                    ui.label(egui::RichText::new(if connected { "true" } else { "false" }).weak());
                });

                ui.separator();
                let busses = ctx
                    .devices
                    .iter()
                    .filter_map(|r| {
                        let Ok(dev) = r.value().lock() else {
                            return None;
                        };
                        dev.viiper_device.as_ref().map(|v| v.device.bus_id)
                    })
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

                let is_localhost = ctx
                    .viiper_address
                    .map(|addr| addr.ip().is_loopback())
                    .unwrap_or(false);

                ui.separator();
                ui.add_enabled_ui(!is_localhost, |ui| {
                    let mut enabled = ctx.keyboard_mouse_emulation;
                    if ui
                        .checkbox(&mut enabled, "Keyboard/mouse emulation")
                        .changed()
                    {
                        if let Err(e) = sdl_loop::get_event_sender().push_custom_event(
                            HandlerEvent::SetKbmEmulation {
                                enabled,
                                initialize: false,
                            },
                        ) {
                            tracing::error!(
                                "Failed to send SetKbmEmulationEnabled event to SDL loop: {}",
                                e
                            );
                        }
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
                            "KB/M emulation requires the SISR window to be in focus",
                        )
                        .weak()
                        .small(),
                    );

                    if ctx.keyboard_mouse_emulation {
                        ui.separator();
                        ui.label(egui::RichText::new("KB/M VIIPER devices").strong());

                        for (label, wanted_type) in
                            [("⌨ Keyboard", "keyboard"), ("🖱 Mouse", "mouse")]
                        {
                            let r = ctx.devices.iter().find(|r| {
                                let Ok(dev) = r.value().lock() else {
                                    return false;
                                };
                                dev.viiper_type.clone().unwrap_or("N/A".to_string()) == wanted_type
                            });
                            egui::CollapsingHeader::new(label)
                                .default_open(true)
                                .id_salt(format!("kbm_viiper_{wanted_type}"))
                                .show(ui, |ui| {
                                    let Some(r) = r else {
                                        return;
                                    };

                                    let Ok(dev) = r.value().lock() else {
                                        tracing::error!(
                                            "Failed to lock Device mutex for VIIPER KB/M info"
                                        );
                                        return;
                                    };

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(egui::RichText::new("Connected:").strong());
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}",
                                                dev.viiper_device.is_some()
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
                                                        viiper_dev.device.bus_id
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
                                                        viiper_dev.device.dev_id.to_string(),
                                                    )
                                                    .weak(),
                                                );
                                            });
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(egui::RichText::new("Type:").strong());
                                                ui.label(
                                                    egui::RichText::new(
                                                        viiper_dev.device.r#type.to_string(),
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
                                                        viiper_dev.device.vid
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
                                                        viiper_dev.device.pid
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
