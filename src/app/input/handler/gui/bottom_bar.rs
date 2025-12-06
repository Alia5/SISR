use std::sync::{Arc, Mutex};

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Align2, Area, FontId, TextStyle, Vec2};
use sdl3::event::EventSender;
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

    pub fn draw(
        &mut self,
        state: &mut State,
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        ctx: &egui::Context,
    ) {
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
                                ..Default::default()
                            },
                        );

                        job.append(
                            format!("\n{}", item.title).as_str(),
                            0.0,
                            TextFormat {
                                font_id: ui.style().text_styles[&TextStyle::Heading].clone(),
                                ..Default::default()
                            },
                        );

                        let response = stacked_button(ui, job, item.open, Vec2::new(24.0, 12.0));
                        if response.clicked() {
                            item.open = !item.open;
                            trace!("Toggled bottom bar item '{}': {}", item.title, item.open);
                        }
                    }
                });
            });

        for item in &mut self.items {
            if item.open {
                (item.render)(state, sdl_waker.clone(), ctx, &mut item.open);
            }
        }
    }
}
