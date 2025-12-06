use std::sync::{Arc, Mutex, OnceLock};

use anyhow::Error;
use egui::{Align2, Sense, Vec2};
use tracing::error;
use winit::event_loop::EventLoopProxy;

use crate::app::window::RunnerEvent;

pub static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub type GuiCallback = Box<dyn Fn(&egui::Context) + Send + Sync + 'static>;
pub type Callback = Box<dyn Fn() + Send + Sync + 'static>;

pub fn push_dialog(dialog: Dialog) -> Result<(), Error> {
    let registry = REGISTRY
        .get()
        .ok_or_else(|| Error::msg("Dialog registry not initialized"))?;
    registry.push_dialog(dialog);
    Ok(())
}

pub fn pop_dialog() -> Result<Arc<Dialog>, Error> {
    let registry = REGISTRY
        .get()
        .ok_or_else(|| Error::msg("Dialog registry not initialized"))?;
    registry
        .pop_dialog()
        .ok_or_else(|| Error::msg("No dialog to pop"))
}

#[derive(Debug, Default)]
pub struct Registry {
    dialogs: Mutex<Vec<Arc<Dialog>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
}

#[derive(Default)]
pub struct Dialog {
    pub title: String,
    pub message: String,
    pub positive_label: Option<String>,
    pub negative_label: Option<String>,
    //
    pub draw_callback: Option<GuiCallback>,
    pub on_positive: Option<Callback>,
    pub on_negative: Option<Callback>,
}

impl std::fmt::Debug for Dialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialog")
            .field("title", &self.title)
            .field("message", &self.message)
            .field("positive_label", &self.positive_label)
            .field("negative_label", &self.negative_label)
            .finish()
    }
}

impl Registry {
    pub fn new(winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>) -> Self {
        Self {
            dialogs: Mutex::new(Vec::new()),
            winit_waker,
        }
    }

    pub fn is_empty(&self) -> bool {
        let Ok(guard) = self.dialogs.lock() else {
            error!("Failed to acquire dialog registry lock to check if empty");
            return true;
        };
        guard.is_empty()
    }
    pub fn snapshot_dialogs(&self) -> Vec<Arc<Dialog>> {
        let Ok(guard) = self.dialogs.lock() else {
            error!("Failed to acquire dialog registry lock to snapshot dialogs");
            return Vec::new();
        };
        guard.clone()
    }

    pub fn push_dialog(&self, dialog: Dialog) {
        let Ok(mut guard) = self.dialogs.lock() else {
            error!("Failed to acquire dialog registry lock to pop dialog");
            return;
        };
        guard.push(Arc::new(dialog));
        drop(guard);

        let Ok(waker_guard) = self.winit_waker.lock() else {
            error!("Failed to acquire winit waker lock to request dialog redraw");
            return;
        };
        if let Some(proxy) = &*waker_guard
            && let Err(e) = proxy.send_event(RunnerEvent::DialogPushed())
        {
            error!(
                "Failed to request dialog redraw after pushing dialog: {}",
                e
            );
        }
    }

    pub fn pop_dialog(&self) -> Option<Arc<Dialog>> {
        let Ok(mut guard) = self.dialogs.lock() else {
            error!("Failed to acquire dialog registry lock to pop dialog");
            return None;
        };
        let dlg = guard.pop();
        drop(guard);

        let Ok(waker_guard) = self.winit_waker.lock() else {
            error!("Failed to acquire winit waker lock to request dialog redraw");
            return dlg;
        };
        if let Some(proxy) = &*waker_guard
            && let Err(e) = proxy.send_event(RunnerEvent::DialogPopped())
        {
            error!(
                "Failed to request dialog redraw after popping dialog: {}",
                e
            );
        }
        dlg
    }
}

impl Dialog {
    pub fn new<T: Into<String>, M: Into<String>>(title: T, message: M) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            ..Default::default()
        }
    }

    pub fn new_ok<T: Into<String>, M: Into<String>, F: Fn() + Send + Sync + 'static>(
        title: T,
        message: M,
        on_positive: F,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            positive_label: Some("OK".to_string()),
            on_positive: Some(Box::new(on_positive)),
            ..Default::default()
        }
    }

    pub fn new_ok_cancel<
        T: Into<String>,
        M: Into<String>,
        F1: Fn() + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
    >(
        title: T,
        message: M,
        on_positive: F1,
        on_negative: F2,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            positive_label: Some("OK".to_string()),
            negative_label: Some("Cancel".to_string()),
            on_positive: Some(Box::new(on_positive)),
            on_negative: Some(Box::new(on_negative)),
            ..Default::default()
        }
    }

    pub fn new_yes_no<
        T: Into<String>,
        M: Into<String>,
        F1: Fn() + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
    >(
        title: T,
        message: M,
        on_positive: F1,
        on_negative: F2,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            positive_label: Some("Yes".to_string()),
            negative_label: Some("No".to_string()),
            on_positive: Some(Box::new(on_positive)),
            on_negative: Some(Box::new(on_negative)),
            ..Default::default()
        }
    }

    pub fn draw(&self, ctx: &egui::Context) {
        self.draw_backdrop(ctx);
        egui::Window::new(egui::WidgetText::from(self.title.clone()))
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .max_size(ctx.available_rect().size() - Vec2::splat(24.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(self.title.clone());
                    ui.separator();
                    egui::ScrollArea::both().auto_shrink(true).show(ui, |ui| {
                        if let Some(callback) = &self.draw_callback {
                            (callback)(ctx);
                        } else {
                            ui.label(&self.message);
                        }
                    });
                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let pos_label = self.positive_label.as_deref().unwrap_or("OK");
                        let pos_button =
                            egui::Button::new(pos_label).fill(ui.visuals().selection.bg_fill);
                        if ui.add(pos_button).clicked() {
                            if let Some(pos_callback) = &self.on_positive {
                                (pos_callback)();
                            }
                            _ = pop_dialog().inspect_err(|e| {
                                error!("Failed to pop dialog after positive button clicked: {}", e);
                            })
                        }
                        if let Some(neg_label) = &self.negative_label
                            && ui.button(neg_label).clicked()
                        {
                            if let Some(neg_callback) = &self.on_negative {
                                (neg_callback)();
                            }
                            _ = pop_dialog().inspect_err(|e| {
                                error!("Failed to pop dialog after negative button clicked: {}", e);
                            })
                        }
                    });
                });
            });
    }

    fn draw_backdrop(&self, ctx: &egui::Context) {
        egui::Area::new(egui::Id::new(format!("dialog_backdrop_{}", self.title)))
            .fixed_pos(egui::Pos2::ZERO)
            .interactable(true)
            .show(ctx, |ui| {
                let screen_rect = ctx.viewport_rect();
                let response = ui.allocate_rect(screen_rect, egui::Sense::click());
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128),
                );
                response.clicked();
            });
    }
}
