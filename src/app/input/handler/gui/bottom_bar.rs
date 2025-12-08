use std::sync::{Arc, Mutex};

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Align2, Area, FontId, TextStyle, Vec2};
use sdl3::event::EventSender;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::app::gui::stacked_button::stacked_button;
use crate::app::input::handler::State;

type RenderFn =
    fn(&mut State, sdl_waker: Arc<Mutex<Option<EventSender>>>, &egui::Context, &mut bool);

pub struct BarItem {
    pub title: &'static str,
    pub icon: &'static str,
    pub open: bool,
    pub render: RenderFn,
}

pub struct BottomBar {
    pub items: Vec<BarItem>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct BottomBarState {
    open_items: Vec<String>,
}

impl BottomBar {
    pub fn new() -> Self {
        Self {
            items: vec![
                BarItem {
                    title: "Gamepads",
                    icon: "🎮",
                    open: false,
                    render: super::controller_info::draw,
                },
                BarItem {
                    title: "VIIPER Info",
                    icon: "🐍",
                    open: false,
                    render: super::viiper_info::draw,
                },
                BarItem {
                    title: "Steam Stuff",
                    icon: "🚂",
                    open: false,
                    render: super::steam_stuff::draw,
                },
            ],
        }
    }

    fn load_state(&mut self, ctx: &egui::Context) {
        let state =
            ctx.data_mut(|d| d.get_persisted::<BottomBarState>(egui::Id::new("bottom_bar_state")));
        if let Some(state) = state {
            for item in &mut self.items {
                item.open = state.open_items.contains(&item.title.to_string());
            }
        }
    }

    fn save_state(&self, ctx: &egui::Context) {
        let state = BottomBarState {
            open_items: self
                .items
                .iter()
                .filter(|i| i.open)
                .map(|i| i.title.to_string())
                .collect(),
        };
        ctx.data_mut(|d| d.insert_persisted(egui::Id::new("bottom_bar_state"), state));
    }

    pub fn draw(
        &mut self,
        state: &mut State,
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        ctx: &egui::Context,
    ) {
        self.load_state(ctx);

        let mut state_changed = false;

        Area::new("input_gui_bottom_bar".into())
            .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -16.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(24.0, 0.0);

                    for item in &mut self.items {
                        let mut job = LayoutJob {
                            halign: Align::Center,
                            ..Default::default()
                        };
                        job.append(
                            item.icon,
                            0.0,
                            TextFormat {
                                font_id: FontId::new(56.0, egui::FontFamily::Proportional),
                                color: ui.style().visuals.text_color(),
                                ..Default::default()
                            },
                        );

                        job.append(
                            format!("\n{}", item.title).as_str(),
                            0.0,
                            TextFormat {
                                font_id: ui.style().text_styles[&TextStyle::Heading].clone(),
                                color: ui.style().visuals.text_color(),
                                ..Default::default()
                            },
                        );

                        let response = stacked_button(ui, job, item.open, Vec2::new(24.0, 12.0));
                        if response.clicked() {
                            item.open = !item.open;
                            state_changed = true;
                            trace!("Toggled bottom bar item '{}': {}", item.title, item.open);
                        }
                    }
                });
            });

        for item in &mut self.items {
            if item.open {
                (item.render)(state, sdl_waker.clone(), ctx, &mut item.open);
                if !item.open {
                    state_changed = true;
                }
            }
        }

        if state_changed {
            self.save_state(ctx);
        }
    }
}
