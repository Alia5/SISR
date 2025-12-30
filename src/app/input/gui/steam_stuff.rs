use egui::{Button, Id, RichText, Vec2};
use tracing::warn;

use crate::app::core::get_tokio_handle;
use crate::app::input::context::Context;
use crate::app::steam_utils::binding_enforcer::binding_enforcer;
use crate::app::steam_utils::cef_debug::inject::get_ws_server_port;
use crate::app::steam_utils::util::{
    launched_in_steam_game_mode, launched_via_steam, open_controller_config,
};
use crate::app::window;
use crate::config::CONFIG;

pub fn draw(ctx: &Context, ectx: &egui::Context, open: &mut bool) {
    if !*open {
        return;
    }

    egui::Window::new("🚂 Steam Stuff")
        .id(Id::new("steam_stuff"))
        .default_pos(ectx.available_rect().center() - Vec2::new(210.0, 200.0))
        .default_size(Vec2::new(360.0, 260.0))
        .collapsible(false)
        .resizable(true)
        .open(open)
        .show(ectx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let Ok(mut enforcer) = binding_enforcer().lock() else {
                    warn!("Failed to acquire binding enforcer lock for Steam Stuff GUI");
                    return;
                };

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
                            if ctx.steam_overlay_open {
                                "Open"
                            } else {
                                "Closed"
                            }
                            .to_string(),
                        )
                        .weak(),
                    );
                });

                let mut continuous = CONFIG
                    .read()
                    .ok()
                    .and_then(|cfg_opt| cfg_opt.as_ref().and_then(|cfg| cfg.window.continous_draw))
                    .unwrap_or(false);

                if ui
                    .checkbox(&mut continuous, "Draw continuously to window")
                    .changed()
                {
                    // FUCK CLIPPY
                    if let Ok(mut config_guard) = CONFIG.write()
                        && let Some(cfg) = config_guard.as_mut()
                    {
                        cfg.window.continous_draw = Some(continuous);
                        window::set_continuous_redraw(continuous);
                    }
                }

                ui.separator();

                let has_app_id = enforcer.app_id().is_some() && enforcer.app_id() != Some(0);
                let mut active = enforcer.is_active();
                ui.separator();
                ui.collapsing(
                    RichText::new("Steam Input Config").strong().size(18.0),
                    |ui| {
                        ui.add_enabled_ui(has_app_id, |ui| {
                            let Some(app_id) = enforcer.app_id() else {
                                return;
                            };

                            let launched_in_game_mode = launched_in_steam_game_mode();
                            ui.add_enabled_ui(!launched_in_game_mode, |ui| {
                                if launched_in_game_mode {
                                    ui.label(
                                        RichText::new(
                                            "Will not force config: Launched in Steam Gaming Mode",
                                        )
                                        .weak(),
                                    );
                                }
                                if ui.checkbox(&mut active, "Force Config").changed() {
                                    if active {
                                        enforcer.activate_with_appid(app_id);
                                    } else {
                                        enforcer.deactivate();
                                    }
                                }
                                ui.style_mut().spacing.button_padding = Vec2::new(12.0, 6.0);
                                let btn = Button::new("🛠 Open Configurator").selected(true);
                                if ui.add(btn).clicked() {
                                    get_tokio_handle().spawn(open_controller_config(app_id));
                                }
                                ui.reset_style();
                            });
                        });
                        ui.separator();
                    },
                );
                ui.collapsing("CEF Stuff", |ui| {
                    if let Some(port) = get_ws_server_port() {
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
