use std::{
    process::ExitCode,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use sdl3_sys::mouse::{SDL_HideCursor, SDL_ShowCursor};
use wgpu::CurrentSurfaceTexture;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{CursorGrabMode, Fullscreen, Window, WindowAttributes, WindowId, WindowLevel},
};

use crate::{
    app::{
        assets::ICON_BYTES,
        input::context::Context,
        steam,
        tray::{self, event::TrayEvent},
        window::{
            event::{EVENT_SENDER, WindowRunnerEvent},
            gfx::Gfx,
            handler::{enter_capture_mode, hide_window, invalidate_svelte_state, overlay_state_changed, quit, redraw, router::EventRouter, set_kbm_cursor_grab, show_window, toggle_ui},
            input_forward::InputForward,
            webview::WebView,
        },
    },
    config::get_config,
};

pub struct WindowRunner {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    continuous_draw: bool,
    passthrough_window: bool,
    router: EventRouter,
    input_forwarder: InputForward,
    webview: Option<WebView>,
    overlay_open: bool,
    previous_webview_visibility: bool,
    previous_continuous_draw: bool,
    previous_passthrough_window: bool,
    ctx: Arc<std::sync::Mutex<Context>>,
}

impl WindowRunner {


    pub fn new(ctx: Arc<std::sync::Mutex<Context>>) -> Self {
        let cfg = get_config();

        let mut router = EventRouter::new();
        router.register(Arc::new(quit::Handler {}));
        router.register(Arc::new(redraw::Handler {}));
        router.register(Arc::new(toggle_ui::Handler::default()));
        router.register(Arc::new(show_window::Handler::default()));
        router.register(Arc::new(hide_window::Handler::default()));
        router.register(Arc::new(overlay_state_changed::Handler::default()));
        router.register(Arc::new(invalidate_svelte_state::Handler::default()));
        router.register(Arc::new(enter_capture_mode::Handler::default()));
        router.register(Arc::new(set_kbm_cursor_grab::Handler::default()));

        Self {
            window: None,
            gfx: None,
            continuous_draw: cfg.window.continuous_draw.unwrap_or(false),
            passthrough_window: false,
            router,
            input_forwarder: InputForward::new(),
            webview: None,
            overlay_open: false,
            previous_webview_visibility: false,
            ctx,
            previous_continuous_draw: false,
            previous_passthrough_window: false,
        }
    }

    pub fn run(&mut self) -> ExitCode {
        let event_loop = EventLoop::<WindowRunnerEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);

        EVENT_SENDER
            .set(Arc::new(event_loop.create_proxy()))
            .expect("Failed to set global event sender");

        match event_loop.run_app(self) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("Error during event loop: {}", e);
                ExitCode::from(1)
            }
        }
    }

    pub fn is_kbm_enabled(&self) -> bool {
        self.ctx
            .lock()
            .ok()
            .map(|c| c.keyboard_mouse_emulation)
            .unwrap_or(false)
    }

    pub fn set_kbm_emulation_enabled(&mut self, enabled: bool) {
        if let Ok(mut context) = self.ctx.lock() {
            context.keyboard_mouse_emulation = enabled;
        } else {
            tracing::error!("Failed to lock Context mutex");
            return;
        }
        let passthrough = get_config().window.fullscreen.unwrap_or(false)
            && !enabled
            && !self.webview.as_ref().is_some_and(|v| v.is_visible());
    }

    pub fn get_window(&self) -> Option<Arc<Window>> {
        self.window.clone()
    }

    pub fn get_webview(&self) -> Option<&WebView> {
        self.webview.as_ref()
    }

    pub fn get_webview_mut(&mut self) -> Option<&mut WebView> {
        self.webview.as_mut()
    }

    pub fn set_overlay_open(&mut self, open: bool) {
        self.overlay_open = open;
    }

    pub fn set_webview_visible(&mut self, visible: bool) {
        self.previous_webview_visibility = self.webview.as_ref().is_some_and(|v| v.is_visible());
        if visible {
            if let Some(webview) = self.webview.as_mut() {
                webview.show();
            }
        } else {
            if let Some(webview) = self.webview.as_mut() {
                webview.hide();
            }
        }
    }

    pub fn restore_webview_visibility(&mut self) {
        if self.previous_webview_visibility {
            if let Some(webview) = self.webview.as_mut() {
                webview.show();
            }
        } else {
            if let Some(webview) = self.webview.as_mut() {
                webview.hide();
            }
        }
    }

    pub fn set_continuous_draw(&mut self, enable: bool) {
        self.previous_continuous_draw = self.continuous_draw;
        self.continuous_draw = enable;
    }


    pub fn restore_continuous_draw(&mut self) {
        self.continuous_draw = self.previous_continuous_draw;
        let sender = crate::app::window::event::get_event_sender();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            let _ = sender.send_event(crate::app::window::event::WindowRunnerEvent::Redraw());
        });
    }

    pub fn set_passthrough(&mut self, enable: bool) {
        let Some(window) = &self.window else {
            return;
        };
        // Don't enable passthrough if overlay is open
        let guarded_enable = enable && !self.overlay_open;
        
        if self.passthrough_window == guarded_enable {
            return;
        }
        self.previous_passthrough_window = self.passthrough_window;

        self.passthrough_window = guarded_enable;
        let _ = window.set_cursor_hittest(!guarded_enable);
    }

    pub fn restore_passthrough(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        self.passthrough_window = self.previous_passthrough_window;
        let _ = window.set_cursor_hittest(!self.passthrough_window);
    }

    pub fn update_cursor_visibility(&self) {
        let hide = !self.webview.as_ref().is_some_and(|v| v.is_visible()) && self.is_kbm_enabled();

        if let Some(window) = &self.window {
            window.set_cursor_visible(!hide);
        }

        unsafe {
            if hide {
                let _ = SDL_HideCursor();
            } else {
                let _ = SDL_ShowCursor();
            }
        }
    }

    fn render(&mut self) -> Option<Duration> {
        let Some(gfx) = &self.gfx else {
            return None;
        };

        let frame = match gfx.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }
            _ => return None,
        };

        let _view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("window_clear_encoder"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("window_clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        gfx.queue.submit(std::iter::once(encoder.finish()));

        frame.present();

        None
    }
}

impl ApplicationHandler<WindowRunnerEvent> for WindowRunner {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        _event: DeviceEvent,
    ) {
        // TODO: is this even needed?

        // if self.ui_visible {
        //     return;
        // }
        // if !is_kbm_emulation_enabled() {
        //     return;
        // }

        // if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
        //     let dx = dx as f32;
        //     let dy = dy as f32;
        //     if dx != 0.0 || dy != 0.0 {
        //         self.try_push_kbm_event(HandlerEvent::KbmPointerEvent(
        //             kbm_events::KbmPointerEvent::motion(dx, dy),
        //         ));
        //     }
        // }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let initially_visible = get_config().window.create.unwrap_or(false);
        let fullscreen = get_config().window.fullscreen.unwrap_or(true);

        let icon = image::load_from_memory(ICON_BYTES).ok().and_then(|img| {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            winit::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
        });

        #[allow(unused_mut)]
        let mut window_attrs = WindowAttributes::default()
            .with_title("SISR")
            .with_transparent(true)
            .with_visible(true)
            .with_window_icon(icon.clone());

        // fuck clippy
        if fullscreen {
            window_attrs = window_attrs
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .with_decorations(false);
        } else {
            window_attrs =
                window_attrs.with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        }
        if steam::util::launched_in_steam_game_mode() {
            tracing::info!("Launched in Steam game mode, fixing window shenanigans...");
            let monitor = event_loop
                .primary_monitor()
                .or_else(|| event_loop.available_monitors().next());
            if let Some(monitor) = monitor {
                let size = monitor.size();
                window_attrs = window_attrs.with_inner_size(size);
                tracing::debug!("Setting window size to {:?}", size);
            } else {
                window_attrs = window_attrs
                    .with_fullscreen(Some(Fullscreen::Borderless(None)))
                    .with_decorations(false);
                tracing::debug!("Could not get monitor info, setting borderless fullscreen");
            }
        }

        #[cfg(windows)]
        {
            use winit::platform::windows::WindowAttributesExtWindows;

            window_attrs = window_attrs.with_taskbar_icon(icon);
            window_attrs = window_attrs.with_clip_children(false);

            if window_attrs.transparent {
                tracing::trace!("Disabling redirection bitmap for transparency on Windows");
                window_attrs = window_attrs.with_no_redirection_bitmap(true);
            }
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attrs)
                .expect("Failed to create window"),
        );

        let window_clone = window.clone();
        if fullscreen {
            window.set_window_level(WindowLevel::AlwaysOnTop);
            #[cfg(windows)]
            set_dwm_passive_update_mode(window.as_ref());
        }

        let mut webview = WebView::new(window.clone());
        if !fullscreen && initially_visible {
            webview.show();
        } else {
            webview.hide();
        }

        tracing::trace!("Window created, visible: {}", initially_visible);
        let gfx = pollster::block_on(Gfx::new(window.clone()));

        if let Err(e) =
            tray::event::get_event_sender().send(TrayEvent::SetWindowState(webview.is_visible()))
        {
            tracing::error!("Failed to send SetWindowState event: {:?}", e);
        }

        self.gfx = Some(gfx);
        self.window = Some(window);

        let passthrough = fullscreen && !webview.is_visible() && !self.is_kbm_enabled();
        self.set_passthrough(passthrough);
        if !webview.is_visible()
            && self.is_kbm_enabled()
            && let Some(window) = &self.window
        {
            // CLIPPY!!!!
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                tracing::warn!("Failed to confine cursor to window: {e}");
            }
        }
        self.update_cursor_visibility();

        self.webview = Some(webview);

        self.previous_webview_visibility = self.webview.as_ref().is_some_and(|v| v.is_visible());
        self.previous_continuous_draw = self.continuous_draw;
        self.previous_passthrough_window = self.passthrough_window;

        // self.window_ready.notify_waiters();
        // self.window_ready.notify_one();

        let window = window_clone.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            window.set_visible(initially_visible);
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: WindowRunnerEvent) {
        let Some(handler) = self.router.handler_for(&_event) else {
            tracing::warn!("No handler found for event: {:?}", _event);
            return;
        };
        handler.handle_event(self, event_loop, &_event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "linux")]
        if self.webview.is_some() {
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }

        if !self.continuous_draw {
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
        if !self.continuous_draw && matches!(event, WindowEvent::RedrawRequested) {
            if let Some(repaint_after) = self.render()
                && let Some(window) = self.window.as_ref()
                && repaint_after < Duration::MAX
            {
                // TODO: Handle repaint_after properly
                window.request_redraw();
            }
            return;
        }

        match &event {
            WindowEvent::CloseRequested => {
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

        let capture_forward = !self.webview.as_ref().is_some_and(|v| v.is_visible()) 
            && self.is_kbm_enabled();

        self.input_forwarder
            .handle_input(&self.window, event_loop, &event, capture_forward);
    }
}

impl Drop for WindowRunner {
    fn drop(&mut self) {
        unsafe {
            let _ = SDL_ShowCursor();
        }

        drop(self.gfx.take());
        drop(self.window.take());
    }
}

// ///

#[cfg(windows)]
fn set_dwm_passive_update_mode(window: &Window) {
    use windows_sys::Win32::{
        Foundation::{E_INVALIDARG, HWND},
        Graphics::Dwm::{DWMWA_PASSIVE_UPDATE_MODE, DwmSetWindowAttribute},
    };
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(window_handle) = window.window_handle() else {
        tracing::warn!("Failed to get window handle for passive update mode");
        return;
    };

    let hwnd = match window_handle.as_raw() {
        RawWindowHandle::Win32(win32) => (win32.hwnd.get() as usize) as HWND,
        _ => {
            tracing::warn!("Expected Win32 window handle for passive update mode");
            return;
        }
    };
    if hwnd.is_null() {
        tracing::warn!("Failed to get HWND for passive update mode");
        return;
    }

    unsafe {
        let passive_update_mode: i32 = 1;
        let result = DwmSetWindowAttribute(
            hwnd,
            DWMWA_PASSIVE_UPDATE_MODE as u32,
            (&passive_update_mode as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        if result != 0 {
            if result == E_INVALIDARG {
                tracing::debug!(
                    "DWMWA_PASSIVE_UPDATE_MODE not supported on this system/window (HRESULT: {:#x})",
                    result
                );
            } else {
                tracing::warn!(
                    "DwmSetWindowAttribute(DWMWA_PASSIVE_UPDATE_MODE) failed with HRESULT: {:#x}",
                    result
                );
            }
        } else {
            tracing::info!("Enabled DWM passive update mode");
        }
    }
}
