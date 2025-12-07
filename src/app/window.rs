use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;

use egui::Context;
use egui_wgpu::Renderer as EguiRenderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State as EguiWinitState;
use tracing::{debug, error, trace, warn};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

#[cfg(windows)]
use winit::platform::windows::WindowAttributesExtWindows;

use crate::app::gui::dispatcher::GuiDispatcher;
use crate::app::gui::{self, dark_theme, dialogs, light_theme};
use crate::config::{self, CONFIG};
use crate::gfx::Gfx;

pub enum RunnerEvent {
    Quit(),
    Redraw(),
    ShowWindow(),
    HideWindow(),
    DialogPushed(),
    DialogPopped(),
}

pub struct WindowRunner {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    egui_ctx: Context,
    egui_winit: Option<EguiWinitState>,
    egui_renderer: Option<EguiRenderer>,
    gui_dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
    winit_waker: Arc<Mutex<Option<winit::event_loop::EventLoopProxy<RunnerEvent>>>>,
    window_ready: Arc<Notify>,
    pre_dialog_window_visible: bool,
}

impl WindowRunner {
    fn get_storage_path() -> Option<std::path::PathBuf> {
        directories::ProjectDirs::from("", "", "SISR")
            .map(|proj_dirs| proj_dirs.data_dir().join("egui_memory.ron"))
    }

    pub fn new(
        winit_waker: Arc<Mutex<Option<winit::event_loop::EventLoopProxy<RunnerEvent>>>>,
        dispatcher: Arc<Mutex<Option<GuiDispatcher>>>,
        window_ready: Arc<Notify>,
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

        gui::dialogs::REGISTRY
            .set(dialogs::Registry::new(winit_waker.clone()))
            .expect("Failed to init dialog registry");

        Self {
            window: None,
            gfx: None,
            egui_ctx: ctx,
            egui_winit: None,
            egui_renderer: None,
            gui_dispatcher: dispatcher,
            // The legend of Zelda: The
            winit_waker,
            window_ready,
            pre_dialog_window_visible: CONFIG
                .get()
                .cloned()
                .expect("Config not set")
                .window
                .create
                .unwrap_or(false),
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
        let Some(registry) = dialogs::REGISTRY.get() else {
            warn!("Dialog registry not initialized");
            return;
        };
        let dialogs = registry.snapshot_dialogs();
        for dialog in dialogs {
            dialog.draw(ctx);
        }
    }

    /// Renders a frame and returns how soon egui wants to repaint (if at all)
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
            if let Ok(guard) = self.gui_dispatcher.lock()
                && let Some(dispatcher) = &*guard
            {
                Self::draw_ui(dispatcher, ctx);
            }
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let initially_visible = config::CONFIG
            .get()
            .cloned()
            .expect("Config not set")
            .window
            .create
            .unwrap_or(false);

        let mut window_attrs = WindowAttributes::default()
            .with_title("SISR")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            .with_transparent(true)
            .with_visible(initially_visible);

        #[cfg(windows)]
        {
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
                    window.focus_window();
                    window.request_redraw();
                } else {
                    trace!("Window is None, cannot show");
                }
            }
            RunnerEvent::HideWindow() => {
                if let Some(window) = &self.window {
                    debug!("hiding window");
                    window.set_visible(false);
                } else {
                    trace!("Window is None, cannot hide");
                }
            }
            RunnerEvent::DialogPushed() => {
                self.pre_dialog_window_visible = self
                    .window
                    .as_ref()
                    .and_then(|w| w.is_visible())
                    .unwrap_or(false);
                if let Some(window) = &self.window {
                    if !self.pre_dialog_window_visible {
                        debug!("Dialog pushed to hidden window, Showing window for dialog");
                        window.set_visible(true);
                        window.focus_window();
                    }
                    trace!("Dialog pushed, requesting redraw");
                    window.request_redraw();
                }
            }
            RunnerEvent::DialogPopped() => {
                if let Some(window) = &self.window {
                    trace!("Dialog popped, requesting redraw");
                    window.request_redraw();
                    if !self.pre_dialog_window_visible {
                        let registry = dialogs::REGISTRY
                            .get()
                            .expect("Dialog registry not initialized");
                        if registry.is_empty() {
                            debug!(
                                "No more dialogs and window was previously hidden, hiding window again"
                            );
                            window.set_visible(false);
                        }
                    }
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::RedrawRequested) {
            if let Some(repaint_after) = self.render()
                && let Some(window) = self.window.as_ref()
                && repaint_after < Duration::MAX
            {
                // TODO: Handle repaint_after properly
                window.request_redraw();
            }
            return;
        }

        let mut egui_consumed = false;

        if let (Some(egui_winit), Some(window)) = (&mut self.egui_winit, &self.window) {
            let response = egui_winit.on_window_event(window.as_ref(), &event);
            if response.repaint {
                window.request_redraw();
            }
            egui_consumed = response.consumed;
        }

        if egui_consumed {
            return;
        }

        match event {
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
            _ => {}
        }
    }
}
