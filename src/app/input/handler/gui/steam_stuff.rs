use std::sync::{Arc, Mutex, atomic::Ordering};

use egui::{Button, Id, RichText, Vec2};
use sdl3::event::EventSender;
use tracing::warn;

use crate::app::input::handler::State;
use crate::app::steam_utils::util::{launched_via_steam, open_steam_url};

pub fn draw(
    state: &mut State,
    _: Arc<Mutex<Option<EventSender>>>,
    ctx: &egui::Context,
    open: &mut bool,
) {
    if !*open {
        return;
    }

    egui::Window::new("🚂 Steam Stuff")
        .id(Id::new("steam_stuff"))
        .default_pos(ctx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_size(Vec2::new(360.0, 260.0))
        .collapsible(false)
        .resizable(true)
        .open(open)
        .show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let enforcer = &mut state.binding_enforcer;

                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Game ID:").strong());
                    ui.label(
                        RichText::new(
                            enforcer
                                .game_id()
                                .map(|id| id.to_string())
                                .unwrap_or("N/A".to_string())
                                .to_string(),
                        )
                        .weak(),
                    );
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("App ID:").strong());
                    ui.label(
                        RichText::new(
                            enforcer
                                .app_id()
                                .map(|id| id.to_string())
                                .unwrap_or("N/A".to_string())
                                .to_string(),
                        )
                        .weak(),
                    );
                });
                let via_steam = launched_via_steam();
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Launch via Steam:").strong());
                    ui.label(
                        RichText::new(if via_steam { "Yes" } else { "No" }.to_string()).weak(),
                    );
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Steam Overlay:").strong());
                    ui.label(
                        RichText::new(
                            if state.steam_overlay_open {
                                "Open"
                            } else {
                                "Closed"
                            }
                            .to_string(),
                        )
                        .weak(),
                    );
                });
                let mut continuous = state.window_continuous_redraw.load(Ordering::Relaxed);
                if ui
                    .checkbox(&mut continuous, "Draw continuously to window")
                    .changed()
                {
                    state
                        .window_continuous_redraw
                        .store(continuous, Ordering::Relaxed);
                }

                ui.separator();

                let has_app_id = enforcer.app_id().is_some();
                let mut active = enforcer.is_active();
                ui.separator();
                ui.collapsing(
                    RichText::new("Steam Input Config").strong().size(18.0),
                    |ui| {
                        ui.add_enabled_ui(has_app_id, |ui| {
                            if ui.checkbox(&mut active, "Force Config").changed() {
                                if active {
                                    enforcer.activate();
                                } else {
                                    enforcer.deactivate();
                                }
                            }
                            ui.style_mut().spacing.button_padding = Vec2::new(12.0, 6.0);
                            let btn = Button::new("🛠 Open Configurator").selected(true);
                            if ui.add(btn).clicked() {
                                let str = &format!(
                                    "steam://controllerconfig/{}",
                                    enforcer.app_id().unwrap()
                                );
                                _ = open_steam_url(str).inspect_err(|e| {
                                    warn!("Failed to open Steam Input Configurator: {}", e)
                                });
                            }
                            ui.reset_style();
                        });
                        ui.separator();
                    },
                );
                ui.collapsing("CEF Stuff", |ui| {
                    if let Some(port) = state.cef_debug_port {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Steam CEF Debug:").strong());
                            ui.label(RichText::new("Enabled").weak());
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("SISR API Port:").strong());
                            ui.label(RichText::new(port.to_string()).weak());
                        });
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("CEF Debug:").strong());
                            ui.label(RichText::new("Not active").weak());
                        });
                    }
                    ui.separator();
                });
            });
        });
}
