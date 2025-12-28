use dashmap::DashMap;
use egui::{CollapsingHeader, Id, RichText, Vec2};

use crate::app::gui::dialogs::{Dialog, push_dialog};
use crate::app::input::context::Context;
use crate::app::input::device::SdlValue;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::sdl_loop;

pub fn draw(ctx: &Context, ectx: &egui::Context, open: &mut bool) {
    egui::Window::new("🎮 Gamepads")
        .id(Id::new("controller_info"))
        .default_pos(ectx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_height(400.0)
        .collapsible(false)
        .default_size(Vec2::new(420.0, 320.0))
        .resizable(true)
        .open(open)
        .show(ectx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let mut devices: Vec<_> = ctx.devices.iter().collect();

                devices.sort_by_key(|r| {
                    let Ok(device) = r.value().lock() else {
                        tracing::error!("Failed to lock Device mutex for sorting in Controller Info GUI");
                        return u64::MAX;
                    };
                    device.id
                });

                for r in devices {
                    let Ok(device) = r.value().lock() else {
                        tracing::error!("Failed to lock Device mutex for Controller Info GUI");
                        continue;
                    };
                    if device.viiper_type == Some("keyboard".to_string()) || device.viiper_type == Some("mouse".to_string() ) {
                        continue;
                    }
                    if !cfg!(debug_assertions)
                        && device.steam_handle == 0 {
                            continue;
                        }

                    let title = device
                        .sdl_devices
                        .iter()
                        .find(|_| true)
                        .and_then(|d| {
                            let infos = if d.gamepad.is_some() {
                                &d.infos.gamepad_infos
                            } else {
                                &d.infos.joystick_infos
                            };
                            infos.get("name").and_then(|r| {
                                match r.value() {
                                    SdlValue::OptString(s) => s.clone(),
                                    _ => None,
                                }
                            })
                        });
                       
                    let title_string = title.unwrap_or_else(|| format!("Device #{}", device.id));
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(
                            title_string.clone(),
                        ));
                        if ui.button("Ignore").clicked() {
                            let device_id = device.id;
                            _ = push_dialog(Dialog::new_yes_no(
                                "Ignore Device", 
                                format!("Are you sure you want to ignore \"{}\"?\nThe device will only reappear once you restart the application.", title_string),
                                 move ||{
                                    if let Err(e) = sdl_loop::get_event_sender().push_custom_event(
                                        HandlerEvent::IgnoreDevice {
                                            device_id
                                        }) {
                                        tracing::error!("Failed to send IgnoreDevice event: {}", e);
                                    }
                                 },
                                 ||{})
                                );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Device ID:").strong());
                                    ui.label(RichText::new(format!("{}", device.id)).weak());
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("SDL IDs:").strong());
                                    ui.label(
                                        RichText::new(
                                            device
                                                .sdl_devices
                                                .iter()
                                                .map(|d| d.id.to_string())
                                                .collect::<Vec<_>>()
                                                .join(", "),
                                        )
                                        .weak(),
                                    );
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Steam Handle:").strong());
                                    ui.label(
                                        RichText::new(format!("{}", device.steam_handle)).weak(),
                                    );
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("SDL Device Count:").strong());
                                    ui.label(
                                        RichText::new(format!("{}", device.sdl_devices.len()))
                                            .weak(),
                                    );
                                });
                            });
                        });
                        ui.separator();

                        let viiper_connect_ui = |ui: &mut egui::Ui| {
                            let enable = if device.viiper_device.is_some() {
                                true
                            } else {
                                device.steam_handle > 0 && ctx.viiper_available
                            };
                            ui.add_enabled_ui(enable, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    if ui
                                        .button(if device.viiper_device.is_some() { "Disconnect" } else { "Connect" })
                                        .clicked()
                                    {
                                        let device_id = device.id;
                                        if let Err(e) = sdl_loop::get_event_sender().push_custom_event(
                                            if device.viiper_device.is_some() {
                                                    HandlerEvent::DisconnectViiperDevice { device_id }
                                                } else {
                                                    HandlerEvent::ConnectViiperDevice { device_id }
                                                }) {
                                            tracing::error!("Failed to send Viiper connect/disconnect event: {}", e);
                                                }
                                    }
                                    let mut selected = device.viiper_type.clone().unwrap_or_else(|| "xbox360".to_string());
                                    let before = selected.clone();
                                    // TODO: extra steamdeck forward crap
                                    egui::ComboBox::from_label("")
                                        .selected_text(selected.clone())
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut selected, "xbox360".to_string(), "xbox360");
                                            ui.selectable_value(&mut selected, "dualshock4".to_string(), "dualshock4");
                                        }
                                    );
                                    if before != selected {
                                        tracing::error!("Changing VIIPER device type not implemented yet");
                                        // TODO;
                                    }
                                });
                                if !enable {
                                    if device.steam_handle == 0 {
                                        ui.label(RichText::new("No Steam handle").weak().small());
                                    } else if !ctx.viiper_available {
                                        ui.label(RichText::new("VIIPER not available").weak().small());
                                    }
                                }
                            });
                        };

                        CollapsingHeader::new(if device.viiper_device.is_some() {
                            "🐍 VIIPER Device 🌐"
                        } else {
                            "🐍 VIIPER Device 🚫"
                        })
                        .id_salt(format!("viiperdev{}", device.id))
                        .default_open(true)
                        .show(ui, |ui| match &device.viiper_device {
                            Some(viiper_dev) => {


                                viiper_connect_ui(ui);

                                CollapsingHeader::new("Connection Info")
                                    .id_salt(format!("viiperdev_conninfo{}", device.id))
                                    .show(ui, |ui|{
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(RichText::new("Bus ID:").strong());
                                            ui.label(
                                                RichText::new(format!("{}", viiper_dev.device.bus_id)).weak(),
                                            );
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(RichText::new("Device ID:").strong());
                                            ui.label(RichText::new(viiper_dev.device.dev_id.to_string()).weak());
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(RichText::new("Type:").strong());
                                            ui.label(RichText::new(viiper_dev.device.r#type.to_string()).weak());
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(RichText::new("Vendor ID:").strong());
                                            ui.label(RichText::new(format!("{:?}", viiper_dev.device.vid)).weak());
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(RichText::new("Product ID:").strong());
                                            ui.label(RichText::new(format!("{:?}", viiper_dev.device.pid)).weak());
                                        });
                                    });
                            }
                            None => {
                                viiper_connect_ui(ui);
                            }
                        });
                    });
                    ui.group(|ui| {
                        for (idx, d) in device.sdl_devices.iter().enumerate() {
                            if d.gamepad.is_some() {
                                ui.collapsing(format!("SDL Gamepad #{}-{}", d.id, idx), |ui| {
                                    render_properties(ui, &d.infos.gamepad_infos);
                                });
                            }
                            if d.joystick.is_some() {
                                ui.collapsing(format!("SDL Joystick #{}-{}", d.id, idx), |ui| {
                                    render_properties(ui, &d.infos.joystick_infos);
                                });
                            }
                        }
                    });
                    ui.separator();
                }
            });
        });
}

fn render_properties(ui: &mut egui::Ui, properties: &DashMap<String, SdlValue>) {
    let mut keys: Vec<_> = properties.iter().map(|entry| entry.key().clone()).collect();
    keys.sort();

    for key in keys {
        let r = properties.get(&key).expect("Key not found");
        let value = r.value();
        match value {
            SdlValue::Nested(nested) => {
                ui.collapsing(format!("📁 {}", key), |ui| {
                    render_properties(ui, nested);
                });
            }
            _ => {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("{}:", key)).strong());
                    ui.label(RichText::new(format!("{}", value)).weak());
                });
            }
        }
    }
}
