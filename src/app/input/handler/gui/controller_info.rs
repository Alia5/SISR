use std::sync::{Arc, Mutex};

use egui::{CollapsingHeader, Id, RichText, Vec2};
use sdl3::event::EventSender;

use crate::app::gui::dialogs::{ Dialog, push_dialog};
use crate::app::input::handler::{HandlerEvent, State};
use crate::app::input::sdl_device_info::SdlValue;

pub fn draw(state: &mut State, sdl_waker: Arc<Mutex<Option<EventSender>>>,  ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("🎮 Gamepads")
        .id(Id::new("controller_info"))
        .default_pos(ctx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_height(400.0)
        .collapsible(false)
        .default_size(Vec2::new(420.0, 320.0))
        .resizable(true)
        .open(open)
        .show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let mut devices: Vec<_> = state.devices.iter().collect();
                devices.sort_by_key(|(_, device)| device.id);
                for (_, device) in devices {
                    if device.viiper_type == "keyboard" || device.viiper_type == "mouse" {
                        continue;
                    }
                    if !cfg!(debug_assertions)
                        && device.steam_handle == 0 {
                            continue;
                        }
                    let title = device
                        .sdl_device_infos
                        .iter()
                        .find(|info| info.is_gamepad && info.properties.contains_key("name"))
                        .and_then(|d| {
                            d.properties.get("name").and_then(|v| match v {
                                SdlValue::OptString(s) => s.clone(),
                                _ => None,
                            })
                        });
                    let title_string = title.unwrap_or_else(|| format!("Device #{}", device.id));
                    let waker_clone = sdl_waker.clone();
                    ui.horizontal(move |ui| {
                        ui.heading(RichText::new(
                            title_string.clone(),
                        ));
                        if ui.button("Ignore").clicked() {
                            let device_id = device.id;
                            _ = push_dialog(Dialog::new_yes_no(
                                "Ignore Device", 
                                format!("Are you sure you want to ignore \"{}\"?\nThe device will only reappear once you restart the application.", title_string),
                                 move ||{
                                    waker_clone.lock().expect("sdl_loop does not exist").as_ref().map(|waker| {
                                        waker.push_custom_event(
                                            HandlerEvent::IgnoreDevice {
                                                device_id
                                            })
                                    });
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
                                                .sdl_ids
                                                .iter()
                                                .map(|id| id.to_string())
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
                                        RichText::new(format!("{}", device.sdl_device_infos.len()))
                                            .weak(),
                                    );
                                });
                            });
                        });
                        ui.separator();

                        let viiper_connect_ui = |ui: &mut egui::Ui| {
                                ui.add_enabled_ui(device.steam_handle > 0, |ui|{
                                    if ui.button(if device.viiper_connected { "Disconnect" } else { "Connect" }).clicked() {
                                        let device_id = device.id;
                                        sdl_waker.clone().lock().expect("sdl_loop does not exist").as_ref().map(|waker| {
                                            waker.push_custom_event(
                                                if device.viiper_connected {
                                                    HandlerEvent::DisconnectViiperDevice { device_id }
                                                } else {
                                                    HandlerEvent::ConnectViiperDevice { device_id }
                                                }
                                            )
                                        });
                                    }
                                });
                        };

                        CollapsingHeader::new(if device.viiper_connected {
                            "🐍 VIIPER Device 🌐"
                        } else {
                            "🐍 VIIPER Device 🚫"
                        })
                        .id_salt(format!("viiperdev{}", device.id))
                        .show(ui, |ui| match &device.viiper_device {
                            Some(viiper_dev) => {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Connected:").strong());
                                    ui.label(
                                        RichText::new(format!("{}", device.viiper_connected))
                                            .weak(),
                                    );
                                });

                                viiper_connect_ui(ui);

                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Bus ID:").strong());
                                    ui.label(
                                        RichText::new(format!("{}", viiper_dev.bus_id)).weak(),
                                    );
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Device ID:").strong());
                                    ui.label(RichText::new(viiper_dev.dev_id.to_string()).weak());
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Type:").strong());
                                    ui.label(RichText::new(viiper_dev.r#type.to_string()).weak());
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Vendor ID:").strong());
                                    ui.label(RichText::new(format!("{:?}", viiper_dev.vid)).weak());
                                });
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new("Product ID:").strong());
                                    ui.label(RichText::new(format!("{:?}", viiper_dev.pid)).weak());
                                });
                            }
                            None => {
                                ui.label("Not connected");
                                viiper_connect_ui(ui);
                            }
                        });
                    });
                    ui.group(|ui| {
                        for (idx, info) in device.sdl_device_infos.iter().enumerate() {
                            ui.collapsing(
                                format!(
                                    "SDL {} #{}-{}",
                                    if info.is_gamepad {
                                        "Gamepad"
                                    } else {
                                        "Joystick"
                                    },
                                    device.id,
                                    idx
                                ),
                                |ui| {
                                    render_properties(ui, &info.properties);
                                },
                            );
                        }
                    });
                    ui.separator();
                }
            });
        });
}

fn render_properties(ui: &mut egui::Ui, properties: &std::collections::HashMap<String, SdlValue>) {
    let mut keys: Vec<_> = properties.keys().collect();
    keys.sort();

    for key in keys {
        let value = &properties[key];
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
