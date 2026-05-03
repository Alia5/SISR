pub mod event;
use std::collections::HashMap;

#[cfg(target_os = "linux")]
use std::rc::Rc;

use tokio::sync::mpsc;
use tracing::{Level, Span};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::app::assets::ICON_BYTES;
use crate::app::tray::event::TrayEvent;
use crate::app::window::event::{WindowRunnerEvent, get_event_sender};

use super::runner::AppRunner;

#[cfg(windows)]
static TRAY_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[cfg(target_os = "linux")]
static GTK_QUIT_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub enum TrayMenuEvent {
    Quit,
    ToggleWindow,
}

struct TrayContext {
    _tray_icon: TrayIcon,
    menu_ids: HashMap<MenuId, TrayMenuEvent>,
    receiver: mpsc::UnboundedReceiver<TrayEvent>,
    toggle_window_item: MenuItem,
}

impl TrayContext {
    fn new() -> Self {
        let icon = load_icon();
        let menu = Menu::new();

        let mut menu_ids = HashMap::new();

        let version_entry =
            MenuItem::new(format!("SISR v{}", env!("CARGO_PKG_VERSION")), false, None);
        menu.append(&version_entry)
            .expect("Failed to add version entry");

        let toggle_window_item = MenuItem::new("Hide UI", true, None);
        menu.append(&toggle_window_item)
            .expect("Failed to add toggle window item");
        menu_ids.insert(toggle_window_item.id().clone(), TrayMenuEvent::ToggleWindow);

        let quit_item = MenuItem::new("Quit", true, None);
        menu.append(&quit_item).expect("Failed to add quit item");
        menu_ids.insert(quit_item.id().clone(), TrayMenuEvent::Quit);

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("SISR")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        let (tx, rx) = mpsc::unbounded_channel();
        event::init(tx);

        Self {
            _tray_icon: tray_icon,
            menu_ids,
            receiver: rx,
            toggle_window_item,
        }
    }

    fn handle_events(&mut self) -> bool {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            match self.menu_ids[&event.id] {
                TrayMenuEvent::Quit => {
                    tracing::info!("Quit requested from tray menu");
                    AppRunner::shutdown();
                    return true;
                }
                TrayMenuEvent::ToggleWindow => {
                    tracing::debug!("Toggle window requested from tray menu");
                    let show = self.toggle_window_item.text() == "Show UI";
                    if let Err(e) = get_event_sender().send_event(WindowRunnerEvent::ToggleUi(show)) {
                        tracing::error!("Failed to send ToggleUi event: {:?}", e);
                    }
                }
            }
        }
        while let Ok(evt) = self.receiver.try_recv() {
            tracing::trace!("Received event: {:?}", evt);
            match evt {
                TrayEvent::SetWindowState(visible) => {
                    if visible {
                        self.toggle_window_item.set_text("Hide UI");
                    } else {
                        self.toggle_window_item.set_text("Show UI");
                    };
                }
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
            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
            tracing::trace!("Posted WM_QUIT to tray thread");
        }
    }

    #[cfg(target_os = "linux")]
    {
        GTK_QUIT_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
        glib::idle_add(|| {
            gtk::main_quit();
            glib::ControlFlow::Break
        });
        tracing::trace!("Set GTK quit flag and scheduled main_quit");
    }
}

pub fn run() {
    let span = tracing::span!(Level::INFO, "tray");
    run_platform(span);
}

#[cfg(windows)]
fn run_platform(span: Span) {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, TranslateMessage, WM_QUIT,
    };

    let thread_id = unsafe { GetCurrentThreadId() };
    TRAY_THREAD_ID.store(thread_id, std::sync::atomic::Ordering::SeqCst);

    let mut ctx = TrayContext::new();

    loop {
        if ctx.handle_events() {
            tracing::event!(parent: &span, Level::DEBUG, "Tray context requested quit, exiting tray loop");
            break;
        }

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 || ret == -1 || msg.message == WM_QUIT {
                tracing::event!(parent: &span, Level::DEBUG, "Received WM_QUIT or error in GetMessageW, exiting tray loop");
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(target_os = "linux")]
fn run_platform(span: Span) {
    use std::cell::RefCell;

    if gtk::init().is_err() {
        tracing::event!(parent: &span, Level::ERROR, "Failed to initialize GTK for tray icon");
        return;
    }

    let ctx = Rc::new(RefCell::new(TrayContext::new()));
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if GTK_QUIT_REQUESTED.load(std::sync::atomic::Ordering::SeqCst) {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        if ctx.borrow_mut().handle_events() {
            gtk::main_quit();
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });

    gtk::main();
}

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
