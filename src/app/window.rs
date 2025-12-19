use egui::text::LayoutJob;
use std::convert::TryFrom;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;

use egui::{Align, Context, FontId, TextFormat, Vec2};
use egui_wgpu::Renderer as EguiRenderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State as EguiWinitState;
use sdl3::event::EventSender;
use tracing::{debug, error, trace, warn};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowAttributes, WindowId, WindowLevel};

#[cfg(windows)]
use winit::platform::windows::WindowAttributesExtWindows;

use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::gui::stacked_button::stacked_button;
use crate::app::gui::{dark_theme, dialogs, light_theme};
use crate::app::input::{handler::HandlerEvent, kbm_events, kbm_winit_map};
use crate::config::{self, CONFIG};
use crate::gfx::Gfx;

pub const ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.ico");

pub enum RunnerEvent {
    Quit(),
    Redraw(),
    ShowWindow(),
    HideWindow(),
    DialogPushed(),
    DialogPopped(),
    ToggleUi(),
    EnterCaptureMode(),
    SetKbmCursorGrab(bool),
    OverlayStateChanged(bool),
}

pub struct WindowRunner {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    egui_ctx: Context,
    egui_winit: Option<EguiWinitState>,
    egui_renderer: Option<EguiRenderer>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    winit_waker: Arc<Mutex<Option<winit::event_loop::EventLoopProxy<RunnerEvent>>>>,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    window_ready: Arc<Notify>,
    pre_dialog_window_visible: bool,
    continuous_redraw: Arc<AtomicBool>,
    last_cursor_pos: Option<(f64, f64)>,
    kbm_emulation_enabled: Arc<AtomicBool>,
    window_visible_shared: Arc<Mutex<bool>>,
    ui_visible_shared: Arc<Mutex<bool>>,
    ui_visible: bool,
    modifiers: winit::keyboard::ModifiersState,
    fullscreen: bool,
    passthrough_active: bool,
    overlay_open: bool,
}

impl WindowRunner {
    fn get_storage_path() -> Option<std::path::PathBuf> {
        directories::ProjectDirs::from("", "", "SISR")
            .map(|proj_dirs| proj_dirs.data_dir().join("egui_memory.ron"))
    }

    pub fn new(
        winit_waker: Arc<Mutex<Option<winit::event_loop::EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
        window_ready: Arc<Notify>,
        continuous_redraw: Arc<AtomicBool>,
        kbm_emulation_enabled: Arc<AtomicBool>,
        window_visible_shared: Arc<Mutex<bool>>,
        ui_visible_shared: Arc<Mutex<bool>>,
    ) -> Self {
        let ctx = Context::default();

        if let Some(storage_path) = Self::get_storage_path()
            && storage_path.exists()
            && let Ok(contents) = std::fs::read_to_string(&storage_path)
            && let Ok(memory) = ron::from_str(&contents)
        {
            ctx.memory_mut(|mem| *mem = memory);
        }

        let light_style = light_theme::style();
        let dark_style = dark_theme::style();

        ctx.all_styles_mut(|style| {
            if style.visuals.dark_mode {
                *style = dark_style.clone();
            } else {
                *style = light_style.clone();
            }
        });

        let cfg = CONFIG
            .read()
            .ok()
            .and_then(|c| c.as_ref().cloned())
            .expect("Config not set");
        Self {
            window: None,
            gfx: None,
            egui_ctx: ctx,
            egui_winit: None,
            egui_renderer: None,
            gui_dispatcher: dispatcher,
            // The legend of Zelda: The
            winit_waker,
            sdl_waker,
            window_ready,
            pre_dialog_window_visible: CONFIG
                .read()
                .ok()
                .and_then(|c| c.as_ref().cloned())
                .expect("Config not set")
                .window
                .create
                .unwrap_or(false),
            continuous_redraw,
            last_cursor_pos: None,
            kbm_emulation_enabled: kbm_emulation_enabled.clone(),
            window_visible_shared,
            ui_visible_shared,
            ui_visible: if cfg.window.fullscreen.unwrap_or(true) {
                false
            } else {
                !kbm_emulation_enabled.load(Ordering::Relaxed)
            },
            modifiers: Default::default(),
            fullscreen: cfg.window.fullscreen.unwrap_or(true),
            passthrough_active: false,
            overlay_open: false,
        }
    }

    fn try_push_kbm_event(&self, ev: HandlerEvent) {
        let Ok(guard) = self.sdl_waker.lock() else {
            return;
        };
        let Some(sender) = guard.as_ref() else {
            return;
        };
        if let Err(e) = sender.push_custom_event(ev) {
            trace!("Failed to push KBM custom event to SDL: {e}");
        }
    }

    fn map_mouse_button(button: winit::event::MouseButton) -> Option<u8> {
        match button {
            winit::event::MouseButton::Left => Some(1),
            winit::event::MouseButton::Middle => Some(2),
            winit::event::MouseButton::Right => Some(3),
            winit::event::MouseButton::Back => Some(4),
            winit::event::MouseButton::Forward => Some(5),
            winit::event::MouseButton::Other(n) => u8::try_from(n).ok(),
        }
    }

    pub fn run(&mut self) -> ExitCode {
        let event_loop = EventLoop::<RunnerEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);

        match self.winit_waker.lock() {
            Ok(mut guard) => {
                let proxy = event_loop.create_proxy();
                *guard = Some(proxy);
            }
            Err(e) => {
                error!("Failed to set winit event loop proxy: {}", e);
            }
        }
        match event_loop.run_app(self) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                error!("Error during event loop: {}", e);
                ExitCode::from(1)
            }
        }
    }

    fn set_passthrough(&mut self, enable: bool) {
        let Some(window) = &self.window else {
            return;
        };
        // Don't enable passthrough if overlay is open
        let enable = enable && !self.overlay_open;

        if self.passthrough_active == enable {
            return;
        }
        self.passthrough_active = enable;
        let _ = window.set_cursor_hittest(!enable);
    }

    fn draw_ui(dispatcher: &GuiDispatcher, ctx: &Context) {
        // egui::Window::new("⚙ EGUI Settings").show(ctx, |ui| {
        //     ctx.settings_ui(ui);
        // });

        // egui::Window::new("🔍 EGUI Inspection").show(ctx, |ui| {
        //     ctx.inspection_ui(ui);
        // });

        egui::Area::new(egui::Id::new("background_panel"))
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    egui::Rect::everything_left_of(f32::INFINITY),
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128),
                );
            });

        dispatcher.draw(ctx);
    }

    fn draw_dialogs(ctx: &Context) {
        let Some(registry) = dialogs::REGISTRY.get() else {
            warn!("Dialog registry not initialized");
            return;
        };
        let dialogs = registry.snapshot_dialogs();
        for dialog in dialogs {
            dialog.draw(ctx);
        }
    }

    fn render(&mut self) -> Option<Duration> {
        let (Some(gfx), Some(window), Some(egui_winit), Some(egui_renderer)) = (
            &self.gfx,
            &self.window,
            &mut self.egui_winit,
            &mut self.egui_renderer,
        ) else {
            return None;
        };

        let frame = match gfx.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return None,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let raw_input = egui_winit.take_egui_input(window.as_ref());

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            if self.ui_visible
                && let Ok(guard) = self.gui_dispatcher.lock()
                && let Some(dispatcher) = &*guard
            {
                Self::draw_ui(dispatcher, ctx);

                if self.fullscreen {
                    egui::Area::new(egui::Id::new("hide_ui_button"))
                        .fixed_pos(egui::Pos2::new(
                            ctx.viewport_rect().max.x,
                            ctx.viewport_rect().min.y,
                        ))
                        .show(ctx, |ui| {
                            let mut job = LayoutJob {
                                halign: Align::Center,
                                ..Default::default()
                            };
                            job.append(
                                "❌",
                                0.0,
                                TextFormat {
                                    font_id: FontId::new(32.0, egui::FontFamily::Proportional),
                                    color: ui.style().visuals.text_color(),
                                    ..Default::default()
                                },
                            );

                            let response = stacked_button(ui, job, true, Vec2::new(24.0, 12.0));
                            if response.clicked() {
                                #[allow(clippy::collapsible_if)]
                                if let Ok(guard) = self.winit_waker.lock()
                                    && let Some(proxy) = guard.as_ref()
                                {
                                    let _ = proxy.send_event(RunnerEvent::ToggleUi());
                                }
                            }
                        });
                    egui::Area::new(egui::Id::new("exit_sisr_button"))
                        .fixed_pos(egui::Pos2::new(
                            ctx.viewport_rect().max.x,
                            ctx.viewport_rect().max.y,
                        ))
                        .show(ctx, |ui| {
                            let mut job = LayoutJob {
                                halign: Align::Center,
                                ..Default::default()
                            };
                            job.append(
                                "Close SISR",
                                0.0,
                                TextFormat {
                                    font_id: FontId::new(32.0, egui::FontFamily::Proportional),
                                    color: ui.style().visuals.text_color(),
                                    ..Default::default()
                                },
                            );

                            let response = stacked_button(ui, job, true, Vec2::new(24.0, 12.0));
                            if response.clicked() {
                                #[allow(clippy::collapsible_if)]
                                if let Ok(guard) = self.winit_waker.lock()
                                    && let Some(proxy) = guard.as_ref()
                                {
                                    let _ = proxy.send_event(RunnerEvent::Quit());
                                }
                            }
                        });
                }
            }

            Self::draw_dialogs(ctx);
        });
        egui_winit.handle_platform_output(window.as_ref(), full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [gfx.config.width, gfx.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        Self::render_egui(
            gfx,
            egui_renderer,
            &view,
            &clipped_primitives,
            &screen_descriptor,
            &full_output.textures_delta,
        );

        frame.present();

        full_output
            .viewport_output
            .get(&self.egui_ctx.viewport_id())
            .map(|vo| vo.repaint_delay)
    }

    fn render_egui(
        gfx: &Gfx,
        renderer: &mut EguiRenderer,
        view: &wgpu::TextureView,
        clipped_primitives: &[egui::ClippedPrimitive],
        screen_descriptor: &ScreenDescriptor,
        textures_delta: &egui::TexturesDelta,
    ) {
        for (id, image_delta) in &textures_delta.set {
            renderer.update_texture(&gfx.device, &gfx.queue, *id, image_delta);
        }

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Egui Encoder"),
            });

        renderer.update_buffers(
            &gfx.device,
            &gfx.queue,
            &mut encoder,
            clipped_primitives,
            screen_descriptor,
        );

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut rpass = rpass.forget_lifetime();
            renderer.render(&mut rpass, clipped_primitives, screen_descriptor);
        }

        gfx.queue.submit(Some(encoder.finish()));

        for id in &textures_delta.free {
            renderer.free_texture(id);
        }
    }
}

impl ApplicationHandler<RunnerEvent> for WindowRunner {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if self.ui_visible {
            return;
        }
        if !self.kbm_emulation_enabled.load(Ordering::Relaxed) {
            return;
        }

        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let dx = dx as f32;
            let dy = dy as f32;
            if dx != 0.0 || dy != 0.0 {
                self.try_push_kbm_event(HandlerEvent::KbmPointerEvent(
                    kbm_events::KbmPointerEvent::motion(dx, dy),
                ));
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let initially_visible = CONFIG
            .read()
            .ok()
            .and_then(|c| c.as_ref().cloned())
            .expect("Config not set")
            .window
            .create
            .unwrap_or(false);

        let icon = image::load_from_memory(ICON_BYTES).ok().and_then(|img| {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            winit::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
        });

        #[allow(unused_mut)]
        let mut window_attrs = WindowAttributes::default()
            .with_title("SISR")
            .with_transparent(true)
            .with_visible(initially_visible)
            .with_window_icon(icon.clone());

        if self.fullscreen {
            window_attrs = window_attrs
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .with_decorations(false);
        } else {
            window_attrs =
                window_attrs.with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        }

        #[cfg(windows)]
        {
            window_attrs = window_attrs.with_taskbar_icon(icon);

            if window_attrs.transparent {
                trace!("Disabling redirection bitmap for transparency on Windows");
                window_attrs = window_attrs.with_no_redirection_bitmap(true);
            }
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        window.set_visible(initially_visible);
        if self.fullscreen {
            window.set_window_level(WindowLevel::AlwaysOnTop);
        }
        trace!("Window created, visible: {}", initially_visible);
        let gfx = pollster::block_on(Gfx::new(window.clone()));

        self.egui_winit = Some(EguiWinitState::new(
            self.egui_ctx.clone(),
            self.egui_ctx.viewport_id(),
            event_loop,
            Some(window.scale_factor() as f32),
            None,
            None,
        ));

        self.egui_renderer = Some(EguiRenderer::new(
            &gfx.device,
            gfx.config.format,
            egui_wgpu::RendererOptions::default(),
        ));

        self.gfx = Some(gfx);
        self.window = Some(window);

        let passthrough = self.fullscreen
            && !self.ui_visible
            && !self.kbm_emulation_enabled.load(Ordering::Relaxed);
        self.set_passthrough(passthrough);
        if !self.ui_visible
            && self.kbm_emulation_enabled.load(Ordering::Relaxed)
            && let Some(window) = &self.window
        {
            // CLIPPY!!!!
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                warn!("Failed to confine cursor to window: {e}");
            }
        }

        self.window_ready.notify_one();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: RunnerEvent) {
        match _event {
            RunnerEvent::Quit() => {
                event_loop.exit();
            }
            RunnerEvent::Redraw() => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            RunnerEvent::ShowWindow() => {
                trace!("ShowWindow event received");
                if let Some(window) = &self.window {
                    debug!("showing window");
                    window.set_visible(true);
                    if self.fullscreen {
                        window.set_window_level(WindowLevel::AlwaysOnTop);
                    }
                    window.focus_window();
                    if !self.ui_visible && self.kbm_emulation_enabled.load(Ordering::Relaxed) {
                        // fuck clippy, there's a difference!
                        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                            warn!("Failed to confine cursor to window: {e}");
                        }
                    }

                    if let Ok(mut guard) = self.window_visible_shared.lock() {
                        *guard = true;
                    }

                    window.request_redraw();
                } else {
                    error!("Window is None, cannot show");
                }
            }
            RunnerEvent::HideWindow() => {
                if let Some(window) = &self.window {
                    debug!("hiding window");
                    _ = window.set_cursor_grab(CursorGrabMode::None);
                    window.set_visible(false);

                    if let Ok(mut guard) = self.window_visible_shared.lock() {
                        *guard = false;
                    }
                } else {
                    error!("Window is None, cannot hide");
                }
            }
            RunnerEvent::ToggleUi() => {
                let Some(window) = self.window.clone() else {
                    return;
                };

                let kbm_emu_enabled = self.kbm_emulation_enabled.load(Ordering::Relaxed);

                if self.ui_visible {
                    self.ui_visible = false;
                    if let Ok(mut g) = self.ui_visible_shared.lock() {
                        *g = false;
                    }
                    let passthrough = self.fullscreen && !kbm_emu_enabled;
                    self.set_passthrough(passthrough);
                    if kbm_emu_enabled {
                        // SCREW CLIPPY!
                        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                            warn!("Failed to confine cursor to window: {e}");
                        }
                    }
                    if self.fullscreen && !self.pre_dialog_window_visible {
                        window.set_visible(false);
                        if let Ok(mut guard) = self.window_visible_shared.lock() {
                            *guard = false;
                        }
                    }
                } else {
                    self.ui_visible = true;
                    if let Ok(mut g) = self.ui_visible_shared.lock() {
                        *g = true;
                    }
                    self.set_passthrough(false);
                    _ = window.set_cursor_grab(CursorGrabMode::None);
                    self.try_push_kbm_event(HandlerEvent::KbmReleaseAll());
                    if !self.pre_dialog_window_visible {
                        window.set_visible(true);
                        if let Ok(mut guard) = self.window_visible_shared.lock() {
                            *guard = true;
                        }
                    }
                    if self.fullscreen {
                        window.set_window_level(WindowLevel::AlwaysOnTop);
                    }
                    window.focus_window();
                }

                window.request_redraw();
            }
            RunnerEvent::EnterCaptureMode() => {
                let Some(window) = self.window.clone() else {
                    return;
                };
                self.ui_visible = false;
                self.set_passthrough(
                    self.fullscreen && !self.kbm_emulation_enabled.load(Ordering::Relaxed),
                );
                if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                    warn!("Failed to confine cursor to window: {e}");
                }
                window.request_redraw();
            }
            RunnerEvent::SetKbmCursorGrab(enabled) => {
                self.kbm_emulation_enabled.store(enabled, Ordering::Relaxed);

                let Some(window) = self.window.clone() else {
                    return;
                };
                if window.is_visible() == Some(false) {
                    return;
                }
                let passthrough = self.fullscreen && !self.ui_visible && !enabled;
                self.set_passthrough(passthrough);
                if !self.ui_visible {
                    if enabled {
                        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                            warn!("Failed to confine cursor to window: {e}");
                        }
                    } else {
                        _ = window.set_cursor_grab(CursorGrabMode::None);
                    }
                } else if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                    warn!("Failed to confine cursor to window: {e}");
                }
            }
            RunnerEvent::OverlayStateChanged(open) => {
                self.overlay_open = open;
                if open {
                    debug!("Steam overlay opened, disabling passthrough");
                    self.set_passthrough(false);
                } else {
                    let should_passthrough = self.fullscreen
                        && !self.ui_visible
                        && !self.kbm_emulation_enabled.load(Ordering::Relaxed);
                    debug!(
                        "Steam overlay closed, restoring passthrough: {}",
                        should_passthrough
                    );
                    self.set_passthrough(should_passthrough);
                }
            }
            RunnerEvent::DialogPushed() => {
                self.set_passthrough(false);
                self.pre_dialog_window_visible = self
                    .window
                    .as_ref()
                    .and_then(|w| w.is_visible())
                    .unwrap_or(false);
                if let Some(window) = &self.window {
                    if !self.pre_dialog_window_visible {
                        debug!("Dialog pushed to hidden window, Showing window for dialog");
                        window.set_visible(true);
                        if self.fullscreen {
                            window.set_window_level(WindowLevel::AlwaysOnTop);
                        }
                        window.focus_window();
                    }
                    trace!("Dialog pushed, requesting redraw");
                    window.request_redraw();
                }
            }
            RunnerEvent::DialogPopped() => {
                let should_restore_passthrough = self.fullscreen
                    && !self.ui_visible
                    && !self.kbm_emulation_enabled.load(Ordering::Relaxed);
                self.set_passthrough(should_restore_passthrough);
                if let Some(window) = &self.window {
                    trace!("Dialog popped, requesting redraw");
                    window.request_redraw();
                    if !self.pre_dialog_window_visible {
                        let window = window.clone();
                        std::thread::spawn(move || {
                            // wait a bit, hack to avoid flicker
                            std::thread::sleep(Duration::from_millis(100));
                            let registry = dialogs::REGISTRY
                                .get()
                                .expect("Dialog registry not initialized");
                            if registry.is_empty() {
                                debug!(
                                    "No more dialogs and window was previously hidden, hiding window again"
                                );
                                window.set_visible(false);
                            }
                        });
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.continuous_redraw.load(Ordering::Relaxed) {
            return;
        }

        static LAST_FRAME_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let last_time = LAST_FRAME_TIME.load(Ordering::Relaxed);
        if last_time != 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            let elapsed = now.saturating_sub(last_time);
            let frame_time = if self.window.as_ref().map(|w| w.has_focus()).unwrap_or(false) {
                16
            } else {
                33
            };
            if elapsed < frame_time {
                std::thread::sleep(Duration::from_millis(frame_time - elapsed));
            }
        }

        LAST_FRAME_TIME.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            Ordering::Relaxed,
        );

        if let Some(repaint_after) = self.render()
            && let Some(window) = self.window.as_ref()
            && repaint_after < Duration::MAX
        {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if !self.continuous_redraw.load(Ordering::Relaxed)
            && matches!(event, WindowEvent::RedrawRequested)
        {
            if let Some(repaint_after) = self.render()
                && let Some(window) = self.window.as_ref()
                && repaint_after < Duration::MAX
            {
                // TODO: Handle repaint_after properly
                window.request_redraw();
            }
            return;
        }

        let any_dialog_shown = dialogs::REGISTRY
            .get()
            .map(|registry| !registry.is_empty())
            .unwrap_or(false);
        if self.ui_visible || any_dialog_shown {
            let mut egui_consumed = false;
            if let (Some(egui_winit), Some(window)) = (&mut self.egui_winit, &self.window) {
                let response = egui_winit.on_window_event(window.as_ref(), &event);
                if response.repaint {
                    window.request_redraw();
                }
                egui_consumed = response.consumed;
            }
            if egui_consumed {
                trace!("egui consumed the event: {:?}", event);
                return;
            }
        }

        match &event {
            WindowEvent::CloseRequested => {
                if let Some(storage_path) = Self::get_storage_path() {
                    if let Some(parent) = storage_path.parent() {
                        _ = std::fs::create_dir_all(parent).inspect_err(|e| {
                            error!("Error creating egui persistance directory: {}", e)
                        });
                    }
                    self.egui_ctx.memory(|mem| {
                        if let Ok(serialized) = ron::to_string(mem) {
                            _ = std::fs::write(&storage_path, serialized).inspect_err(|e| {
                                error!("Error writing egui persistance file: {}", e)
                            });
                        }
                    });
                }
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }

        let capture_forward =
            !self.ui_visible && self.kbm_emulation_enabled.load(Ordering::Relaxed);

        match event {
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::ElementState;
                use winit::keyboard::PhysicalKey;

                if matches!(event.state, ElementState::Pressed)
                    && let PhysicalKey::Code(code) = event.physical_key
                    && code == winit::keyboard::KeyCode::KeyS
                    && self.modifiers.control_key()
                    && self.modifiers.shift_key()
                    && self.modifiers.alt_key()
                {
                    trace!("Toggle UI keybinding pressed");
                    let Some(window) = &self.window else {
                        return;
                    };
                    if self.ui_visible {
                        self.ui_visible = false;
                        if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                            warn!("Failed to confine cursor to window: {e}");
                        }
                    } else {
                        self.ui_visible = true;
                        _ = window.set_cursor_grab(CursorGrabMode::None);
                        self.try_push_kbm_event(HandlerEvent::KbmReleaseAll());
                    }
                    window.request_redraw();
                    return;
                }

                if let PhysicalKey::Code(code) = event.physical_key
                    && let Some(scancode) = kbm_winit_map::keycode_to_sdl_scancode(code)
                {
                    let down = matches!(event.state, ElementState::Pressed);
                    if capture_forward {
                        self.try_push_kbm_event(HandlerEvent::KbmKeyEvent(
                            kbm_events::KbmKeyEvent { scancode, down },
                        ));
                    }
                } else {
                    warn!("Unmapped key event: {:?}", event.physical_key);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                self.last_cursor_pos = Some((x, y));
            }
            WindowEvent::CursorLeft { .. } => {
                self.last_cursor_pos = None;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::ElementState;

                if let Some(btn) = Self::map_mouse_button(button) {
                    let down = matches!(state, ElementState::Pressed);
                    if capture_forward {
                        self.try_push_kbm_event(HandlerEvent::KbmPointerEvent(
                            kbm_events::KbmPointerEvent::button(btn, down),
                        ));
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;

                let (wheel_x, wheel_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };

                if (wheel_x != 0.0 || wheel_y != 0.0) && capture_forward {
                    self.try_push_kbm_event(HandlerEvent::KbmPointerEvent(
                        kbm_events::KbmPointerEvent::wheel(wheel_x, wheel_y),
                    ));
                }
            }
            _ => {}
        }
    }
}
