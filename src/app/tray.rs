use std::sync::{Arc, Mutex};

use sdl3::event::EventSender;
#[cfg(windows)]
use tracing::Span;
use tracing::{Level, error, event, info, span};
#[cfg(target_os = "linux")]
use tracing::{Span, trace};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::event_loop::EventLoopProxy;

use crate::app::gui::dialogs::{self, Dialog};
use crate::app::window::RunnerEvent;

use super::core::App;

const ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.png");

#[cfg(windows)]
static TRAY_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub enum TrayMenuEvent {
    Quit,
    ToggleWindow,
}

struct TrayContext {
    _tray_icon: TrayIcon,
    quit_id: MenuId,
    toggle_window_item: MenuItem,
    toggle_window_id: MenuId,
    window_visible: Arc<Mutex<bool>>,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
}

impl TrayContext {
    fn new(
        sdl_waker: Arc<Mutex<Option<EventSender>>>,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        window_visible: Arc<Mutex<bool>>,
    ) -> Self {
        let icon = load_icon();
        let menu = Menu::new();

        let initial_visible = *window_visible.lock().unwrap();
        let initial_text = if initial_visible {
            "Hide Window"
        } else {
            "Show Window"
        };
        let toggle_window_item = MenuItem::new(initial_text, true, None);
        let toggle_window_id = toggle_window_item.id().clone();
        menu.append(&toggle_window_item)
            .expect("Failed to add toggle window item");

        let quit_item = MenuItem::new("Quit", true, None);
        let quit_id = quit_item.id().clone();
        menu.append(&quit_item).expect("Failed to add quit item");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("SISR")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        Self {
            _tray_icon: tray_icon,
            quit_id,
            toggle_window_item,
            toggle_window_id,
            window_visible,
            sdl_waker,
            winit_waker,
        }
    }

    fn handle_events(&self) -> bool {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_id {
                info!("Quit requested from tray menu");
                App::shutdown(Some(&self.sdl_waker), Some(&self.winit_waker));
                return true;
            }
            if event.id == self.toggle_window_id {
                let Ok(mut guard) = self.window_visible.lock() else {
                    error!("Failed to lock window_visible mutex");
                    return false;
                };
                *guard = !*guard;
                let visible = *guard;
                drop(guard);

                let menu_text = if visible {
                    "Hide Window"
                } else {
                    "Show Window"
                };
                self.toggle_window_item.set_text(menu_text);

                let Ok(winit_guard) = self.winit_waker.lock() else {
                    error!("Failed to lock winit_waker mutex");
                    return false;
                };
                if let Some(proxy) = &*winit_guard {
                    let event = if visible {
                        RunnerEvent::ShowWindow()
                    } else {
                        RunnerEvent::HideWindow()
                    };
                    _ = proxy.send_event(event);
                }
                return false;
            }
        }
        false
    }
}

pub fn shutdown() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
        let thread_id = TRAY_THREAD_ID.load(std::sync::atomic::Ordering::SeqCst);
        if thread_id != 0 {
            use tracing::trace;

            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
            trace!("Posted WM_QUIT to tray thread");
        }
    }

    #[cfg(target_os = "linux")]
    {
        gtk::main_quit();
        trace!("Called gtk::main_quit()");
    }

    #[cfg(target_os = "macos")]
    {
        // TODO: macOS CFRunLoop stop
        tracing::warn!("macOS tray shutdown not yet implemented");
    }
}

pub fn run(
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    window_visible: Arc<Mutex<bool>>,
) {
    let span = span!(Level::INFO, "tray");
    run_platform(span, sdl_waker, winit_waker, window_visible);
}

#[cfg(windows)]
fn run_platform(
    span: Span,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    window_visible: Arc<Mutex<bool>>,
) {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_QUIT,
    };

    let thread_id = unsafe { GetCurrentThreadId() };
    TRAY_THREAD_ID.store(thread_id, std::sync::atomic::Ordering::SeqCst);

    let ctx = TrayContext::new(sdl_waker, winit_waker, window_visible);

    loop {
        if ctx.handle_events() {
            event!(parent: &span, Level::DEBUG, "Tray context requested quit, exiting tray loop");
            break;
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 || msg.message == WM_QUIT {
                event!(parent: &span, Level::DEBUG, "Received WM_QUIT or error in GetMessageW, exiting tray loop");
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_platform(
    span: Span,
    sdl_waker: Arc<Mutex<Option<EventSender>>>,
    winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    window_visible: Arc<Mutex<bool>>,
) {
    if gtk::init().is_err() {
        event!(parent: &span, Level::ERROR, "Failed to initialize GTK for tray icon");
        return;
    }

    let ctx = Arc::new(TrayContext::new(sdl_waker, winit_waker, window_visible));

    glib::idle_add_local(move || {
        if ctx.handle_events() {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });

    gtk::main();
}

#[cfg(target_os = "macos")]
fn run_platform(
    span: Span,
    _sdl_waker: Arc<Mutex<Option<EventSender>>>,
    _winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
    _window_visible: Arc<Mutex<bool>>,
) {
    event!(parent: &span, Level::WARN, "macOS tray icon requires main thread NSApplication event loop - not yet implemented");
}

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
